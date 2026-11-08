#!/usr/bin/python3
"""Stadia bridge. Not installed: parked on 2026-11-06, because a disconnect while it is signed up leaves LG's GATT layer stale until
Bluetooth restarts (docs/research/spikes.md, "Sign-up trigger found"). Kept for the next round on the LG bug.

After the controller reconnects, LG's HID host drops its reports: the controller's Service Changed indication wipes the
host's notification sign-up and the host never signs up again (docs/research/spikes.md, "Stack trace on"). The reports still reach LG's
GATT API, so after each connection this subscribes there. If the TV's own gamepad passes input, it stops. If the TV's gamepad stays silent
while reports arrive, it makes a gamepad of its own through /dev/uhid and feeds them in, so a reconnect works without a re-pair.

Log: /var/lib/homehook/stadia.log (lines start "bridge:"). Off: /var/lib/homehook/no-stadia-bridge.
While it feeds input, /tmp/stadia-bridged holds the TV gamepad's instance (0005:18D1:9400.000C), which stadia.sh reads as working.
"""
import json
import os
import select
import signal
import struct
import subprocess
import time

B = 'luna://com.webos.service.bluetooth2'
SVC = '00001812-0000-1000-8000-00805f9b34fb'
CHR = '00002a4d-0000-1000-8000-00805f9b34fb'
DIR = '/var/lib/homehook'
LOG = DIR + '/stadia.log'
OFF = DIR + '/no-stadia-bridge'
RD_CACHE = DIR + '/stadia-report-map.bin'
FLAG = '/tmp/stadia-bridged'
PIDFILE = '/tmp/stadia-bridge.pid'
# not "Stadia…", so stadia.sh and this script never take the bridge's own gamepad for the controller
NAME_PREFIX = 'Lumen bridge '
REPORT_ID = 3
# the TV's gamepad counts as dead once this many reports came through GATT and none through it
DEAD_AFTER = 3
# discoverServices fails until gatt/connect has made a client for the address (2026-11-06), so connect comes first, and both are retried
DISCOVER_EVERY = 2
DISCOVER_FOR = 180
# the controller's Service Changed comes about 2 s after a connection and wipes every sign-up, this one too (2026-11-06: a sign-up at 0 s got
# nothing). So the first sign-up waits, and while no report has come by either path, it is renewed: often at first, then rarely.
SUBSCRIBE_AFTER = 4
RENEW_EVERY = 5
RENEW_LATER = 30
RENEW_LATER_FROM = 60

# apps run in jails with their own fixed /dev; Moonlight's has only hidraw0 (the TV's dead gamepad), so the bridge's node is added there.
# Moonlight opened it within 3 s of it appearing (2026-11-06).
JAILS = '/var/palm/jail'

UHID_DESTROY, UHID_CREATE2, UHID_INPUT2 = 1, 11, 12
UHID_EVENT_SIZE = 4380
BUS_BLUETOOTH = 5


def log(msg):
    try:
        with open(LOG, 'a') as f:
            f.write(time.strftime('%Y-%m-%d %H:%M:%S') + ' bridge: ' + msg + '\n')
    except OSError:
        pass


def luna(method, params, timeout=25):
    try:
        r = subprocess.run(['luna-send', '-n', '1', B + '/' + method, json.dumps(params)], stdin=subprocess.DEVNULL,
                           stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, timeout=timeout)
        return json.loads(r.stdout.decode() or '{}')
    except (subprocess.TimeoutExpired, ValueError):
        return {}


def read(path):
    try:
        with open(path, 'rb') as f:
            return f.read()
    except OSError:
        return b''


def tv_gamepad():
    """The TV's own gamepad for the controller: (instance, hidraw path, address, report map path), or None."""
    try:
        nodes = sorted(os.listdir('/sys/class/hidraw'))
    except OSError:
        return None
    for n in nodes:
        dev = '/sys/class/hidraw/%s/device' % n
        ev = dict(l.split('=', 1) for l in read(dev + '/uevent').decode(errors='replace').splitlines() if '=' in l)
        if ev.get('HID_NAME', '').startswith('Stadia') and ':000018D1:00009400' in ev.get('HID_ID', '').upper():
            return os.path.basename(os.path.realpath(dev)), '/dev/' + n, ev.get('HID_PHYS', '').lower(), dev + '/report_descriptor'
    return None


class Gamepad:
    """The bridge's own uhid gamepad, a copy of the controller's (same IDs and report map), so Moonlight and the TV treat it the same."""

    def __init__(self, name, phys, rd):
        self.name = name
        self.shared = False
        self.fd = os.open('/dev/uhid', os.O_RDWR | os.O_NONBLOCK)
        ev = struct.pack('<I128s64s64sHHIIII4096s', UHID_CREATE2, name.encode()[:127], phys.encode()[:63], b'',
                         len(rd), BUS_BLUETOOTH, 0x18d1, 0x9400, 0x0111, 0, rd)
        os.write(self.fd, ev.ljust(UHID_EVENT_SIZE, b'\0'))

    def send(self, data):
        os.write(self.fd, struct.pack('<IH4096s', UHID_INPUT2, len(data), data).ljust(UHID_EVENT_SIZE, b'\0'))

    def share_with_jails(self):
        """Adds this gamepad's hidraw node to every app jail that has hidraw nodes. True once done; the node shows up a moment after
        the gamepad is made, so this is retried."""
        for n in sorted(os.listdir('/sys/class/hidraw')):
            if ('HID_NAME=' + self.name).encode() not in read('/sys/class/hidraw/%s/device/uevent' % n):
                continue
            major, minor = (int(x) for x in read('/sys/class/hidraw/%s/dev' % n).decode().split(':'))
            try:
                jails = [os.path.join(JAILS, j, 'dev') for j in os.listdir(JAILS) if os.path.isdir(os.path.join(JAILS, j, 'dev'))]
            except OSError:
                jails = []
            for d in jails:
                try:
                    if not any(f.startswith('hidraw') for f in os.listdir(d)):
                        continue
                    node = os.path.join(d, n)
                    if os.path.exists(node):
                        if os.stat(node).st_rdev == os.makedev(major, minor):
                            continue
                        os.remove(node)
                    os.mknod(node, 0o660 | 0o020000, os.makedev(major, minor))
                    gid = os.stat(os.path.join(d, 'hidraw0')).st_gid if os.path.exists(os.path.join(d, 'hidraw0')) else 0
                    os.chown(node, 0, gid)
                    # mknod's mode passes through the umask, and the app opens the node read-write
                    os.chmod(node, 0o660)
                    log('added /dev/%s to %s' % (n, d))
                except OSError as e:
                    log('cannot add /dev/%s to %s: %s' % (n, d, e))
            self.shared = True
            return True
        return False

    def drain(self):
        # the kernel's start, open, close and output events; output (rumble) is not passed back to the controller
        try:
            os.read(self.fd, UHID_EVENT_SIZE)
        except OSError:
            pass

    def close(self):
        try:
            os.write(self.fd, struct.pack('<I', UHID_DESTROY).ljust(UHID_EVENT_SIZE, b'\0'))
        except OSError:
            pass
        os.close(self.fd)


class Link:
    """One controller connection, from the moment the TV's gamepad appears until it goes."""

    def __init__(self, inst, hidraw, addr, rd_path):
        self.inst, self.addr, self.rd_path = inst, addr, rd_path
        self.since = time.time()
        self.next_try = self.since + SUBSCRIBE_AFTER
        self.subscribed_at = 0
        self.signups = 0
        self.tv_fd = None
        self.tv_bytes = 0
        self.monitor = None
        self.buf = b''
        self.gatt_reports = 0
        self.pad = None
        self.done = False
        try:
            self.tv_fd = os.open(hidraw, os.O_RDONLY | os.O_NONBLOCK)
        except OSError as e:
            log('cannot read %s: %s' % (hidraw, e))

    def fds(self):
        return [f for f in (self.tv_fd, self.monitor.stdout.fileno() if self.monitor else None, self.pad.fd if self.pad else None)
                if f is not None]

    def subscribe(self):
        """Starts the GATT subscription once LG's service discovery works. True when it is running."""
        now = time.time()
        if self.monitor and not self.gatt_reports and not self.tv_bytes and not self.done:
            renew = RENEW_EVERY if now - self.since < RENEW_LATER_FROM else RENEW_LATER
            if now - self.subscribed_at >= renew:
                self.stop_monitor()
                self.next_try = now
        if self.monitor or self.done or now < self.next_try:
            return bool(self.monitor)
        if not self.signups and now - self.since > DISCOVER_FOR:
            log('gave up: LG service discovery failed for %ds' % DISCOVER_FOR)
            self.done = True
            return False
        self.next_try = now + DISCOVER_EVERY
        cid = luna('gatt/connect', {'address': self.addr}).get('clientId')
        if not cid or not luna('gatt/discoverServices', {'address': self.addr}).get('returnValue'):
            return False
        # the address form fails LG's schema; clientId works and picks the input report (instanceId 064, handle 0x0040)
        req = {'clientId': cid, 'service': SVC, 'characteristic': CHR, 'subscribe': True}
        self.monitor = subprocess.Popen(['luna-send', '-i', B + '/gatt/monitorCharacteristic', json.dumps(req)],
                                        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
        os.set_blocking(self.monitor.stdout.fileno(), False)
        self.subscribed_at = now
        self.signups += 1
        if self.signups == 1:
            log('subscribed %ds after the connection (%s)' % (now - self.since, self.inst))
        return True

    def stop_monitor(self):
        if self.monitor:
            self.monitor.kill()
            self.monitor.wait()
            self.monitor = None

    def on_tv_input(self):
        try:
            n = len(os.read(self.tv_fd, 4096))
        except OSError:
            return
        if n and not self.tv_bytes:
            if self.pad:
                log('the TV gamepad came back to life, bridge off (%s)' % self.inst)
                self.stop_pad()
            elif self.monitor:
                log('the TV gamepad works, nothing to bridge (%s)' % self.inst)
            self.stop_monitor()
            self.done = True
        self.tv_bytes += n

    def on_gatt(self):
        try:
            chunk = os.read(self.monitor.stdout.fileno(), 65536)
        except OSError:
            return
        if not chunk:
            log('the GATT subscription ended (%s)' % self.inst)
            self.stop_monitor()
            self.next_try = time.time() + DISCOVER_EVERY
            return
        self.buf += chunk
        *lines, self.buf = self.buf.split(b'\n')
        for line in lines:
            try:
                value = json.loads(line).get('changed', {}).get('value', {}).get('bytes')
            except ValueError:
                continue
            if value:
                self.on_report(bytes([REPORT_ID] + value))

    def on_report(self, data):
        self.gatt_reports += 1
        if self.gatt_reports == 1:
            log('first report %ds after the connection, sign-up %d (%s)' % (time.time() - self.since, self.signups, self.inst))
        if self.tv_bytes or self.done:
            return
        if not self.pad and self.gatt_reports >= DEAD_AFTER:
            rd = read(self.rd_path) or read(RD_CACHE)
            if not rd:
                log('no report map to copy, cannot bridge')
                self.done = True
                return
            try:
                self.pad = Gamepad(NAME_PREFIX + self.inst, self.addr, rd)
            except OSError as e:
                log('cannot make the gamepad: %s' % e)
                self.done = True
                return
            with open(FLAG, 'w') as f:
                f.write(self.inst + '\n')
            log('the TV gamepad is silent, bridging after %d reports (%s)' % (self.gatt_reports, self.inst))
        if self.pad:
            try:
                self.pad.send(data)
            except OSError as e:
                log('gamepad write failed: %s' % e)

    def stop_pad(self):
        if self.pad:
            self.pad.close()
            self.pad = None
        try:
            os.remove(FLAG)
        except OSError:
            pass

    def close(self):
        if self.pad:
            log('controller gone, bridge off (%s, %d reports)' % (self.inst, self.gatt_reports))
        self.stop_monitor()
        self.stop_pad()
        if self.tv_fd is not None:
            os.close(self.tv_fd)


def main():
    with open(PIDFILE, 'w') as f:
        f.write('%d\n' % os.getpid())
    # a flag left by a bridge that was killed would make stadia.sh count a dead connection as working
    try:
        os.remove(FLAG)
    except OSError:
        pass
    signal.signal(signal.SIGTERM, lambda *_: exit(0))
    log('started')
    state = {'link': None}
    try:
        run(state)
    finally:
        # destroys the bridge's gamepad and clears the flag, so a restart starts clean
        if state['link']:
            state['link'].close()


def run(state):
    while True:
        link = state['link']
        if os.path.exists(OFF):
            if link:
                link.close()
                state['link'] = None
            time.sleep(2)
            continue
        gp = tv_gamepad()
        if link and (not gp or gp[0] != link.inst):
            link.close()
            link = state['link'] = None
        if gp and not link:
            link = state['link'] = Link(*gp)
            rd = read(gp[3])
            if rd and rd != read(RD_CACHE):
                with open(RD_CACHE, 'wb') as f:
                    f.write(rd)
        if not link:
            time.sleep(1)
            continue
        link.subscribe()
        fds = link.fds()
        ready = select.select(fds, [], [], 1)[0] if fds else time.sleep(1) or []
        # the TV's gamepad first, so a report that reached both counts as working
        if link.tv_fd in ready:
            link.on_tv_input()
        if link.monitor and link.monitor.stdout.fileno() in ready:
            link.on_gatt()
        if link.pad and link.pad.fd in ready:
            link.pad.drain()
        if link.pad and not link.pad.shared:
            link.pad.share_with_jails()


if __name__ == '__main__':
    try:
        main()
    except Exception as e:  # noqa: BLE001 - logged so a crash is visible in stadia.log
        log('crashed: %r' % e)
        raise
