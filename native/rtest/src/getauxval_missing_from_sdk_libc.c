#include <stdio.h>

unsigned long getauxval(unsigned long type) {
    unsigned long entry[2], value = 0;
    FILE *f = fopen("/proc/self/auxv", "rb");
    if (!f) return 0;
    while (fread(entry, sizeof entry, 1, f) == 1 && entry[0])
        if (entry[0] == type) { value = entry[1]; break; }
    fclose(f);
    return value;
}
