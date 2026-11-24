#!/usr/bin/env python3
import os
import sys
from fontTools.ttLib import TTFont
from fontTools.varLib import instancer

FONTS = sys.argv[1] if len(sys.argv) > 1 else os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), 'app', 'private', 'fonts')
USAGE = 'needs fonttools and brotli (pip, in a throwaway venv); reads sf.woff2 and writes sf-<weight>.ttf beside it, in app/private/fonts or the folder given'
FACES = {'sf-400': (400, 24, False), 'sf-500': (500, 28, False), 'sf-600': (600, 28, False), 'sf-600-clock': (600, 96, True)}


def case_colon(font):
    gsub = font['GSUB'].table
    for record in gsub.FeatureList.FeatureRecord:
        if record.FeatureTag != 'case':
            continue
        for index in record.Feature.LookupListIndex:
            for sub in gsub.LookupList.Lookup[index].SubTable:
                mapping = getattr(sub, 'mapping', None) or getattr(getattr(sub, 'ExtSubTable', None), 'mapping', None) or {}
                if 'colon' in mapping:
                    return mapping['colon']
    return None


if not os.path.exists(os.path.join(FONTS, 'sf.woff2')):
    sys.exit(USAGE)

for name, (weight, opsz, clock) in FACES.items():
    font = instancer.instantiateVariableFont(TTFont(os.path.join(FONTS, 'sf.woff2')), {'wght': weight, 'opsz': opsz})
    font.flavor = None
    font['OS/2'].usWeightClass = weight
    colon = case_colon(font) if clock else None
    if colon:
        for table in font['cmap'].tables:
            if 0x3A in table.cmap:
                table.cmap[0x3A] = colon
    out = os.path.join(FONTS, name + '.ttf')
    font.save(out)
    print(name + '.ttf', weight, 'opsz', opsz, 'case colon' if colon else '', os.path.getsize(out) // 1024, 'KB')
