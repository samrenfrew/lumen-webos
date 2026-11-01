# Privacy

Settings > Privacy turns off LG's ads, tracking and data collection. Each row runs `tools/tv/privacy.sh` as root and shows what the TV
reports. The methods are adapted from Glasshouse (rorygallagher2024/glasshouse, MIT) and cleanyourtv-WebOS; see
`docs/research/prior-art.md`. All of it is **(verify)** in `docs/plans/PLAN.md` until seen working on this TV.

| Row | What it does | Lasts | Undo |
|---|---|---|---|
| LG data agreements | Withdraws the optional agreement flags in the settings service's `eulaStatus`: screen recognition (`acrAllowed`, `acrGdprAllowed`, `acrOnAllowed`), ads based on viewing (`acrAdAllowed`), personalised ads (`customAdAllowed`, `customadsAllowed`), ad cookies, third-party and data-partner sharing, additional data, remote diagnostics, marketing, shopping on live TV and Who.Where.What?. The agreement documents in `eulaInfoNetwork` are marked to match, as Glasshouse's `planConsent` does. Voice, the terms of use, network use and LG Channels' `chpAllowed` are left alone. | Saved, like a menu change | OK again restores the copy saved before the first withdrawal (`/var/lib/homehook/eula.backup.json`), or LG's own menus |
| Limit ad tracking | The TV's `general.lmt` setting | Saved | OK again |
| Live Plus | The TV's `option.livePlus` setting (LG's screen recognition) | Saved | OK again |
| Block LG ad servers | About 90 ad, tracking and Alphonso (screen recognition) names, with the region prefixes LG puts in front of some, in `/var/lib/homehook/hosts-privacy`. `watch.sh` keeps them in `/etc/hosts`. LG's platform stays reachable, so the Content Store and app updates work | While `watch.sh` runs | OK again removes the file and the lines |
| Stop LG tracking services | Masks and stops `uploadd`, `rdxd`, `remotelogger`, `service-logger`, `contentminer` and `nudge` under `/run/systemd/transient`, and binds a do-nothing program over `acr2`, `adoverlay-service` and `sportsalarm`. `60-homehook` does it again at each boot | Until a restart; the boot hook redoes it | OK again, or a restart without the flag |
| Block TV software updates | The Homebrew Channel's own flag, `/var/luna/preferences/webosbrew_block_updates` | From the next restart | OK again |

Left alone on purpose:
- `admanager`: HBO Max and Disney+ wait for it at start (cleanyourtv-WebOS issue 47).
- Voice (`voiceAllowed`, `voiceconductor`).
- Glasshouse's "everything" host tier, which also cuts the Content Store, LG Channels, ThinQ and firmware updates.

Every row needs the TV scripts installed and matching this build (Settings > System > TV scripts). Writing the agreements needs `node`
on the TV; the row says so if it is missing.

To check on the TV: `sh /var/lib/homehook/privacy.sh state`. Each change is logged to `/tmp/homehook.log`.

Unknown on this TV until tried:
- Whether `/etc/hosts` can be written (the row says so if not).
- Whether the agreement flags stay withdrawn after a restart. Glasshouse says they persist; its one report from a sibling TV on webOS
  10.2 says "some settings reported not to apply".
- Whether any app in daily use needs one of the stopped services.
