#!/usr/bin/env python3

import json
import sys
import pyembroidery

from pyembroidery import STITCH, JUMP, COLOR_CHANGE, END

# pyembroidery uses 0.1 mm units.
MM_TO_UNITS = 10

def mm_to_units(point):
    x, y = point[0], point[1]
    return int(round(x * MM_TO_UNITS)), int(round(y * MM_TO_UNITS))

def color_vec_to_hex_int(clr):
    return (round(clr[0] * 255) << 16) + (round(clr[1] * 255) << 8) + (round(clr[2] * 255) << 0)

def convert_json_to_vp3(input_json_path, output_vp3_path):
    with open(input_json_path, "r", encoding="utf-8") as f:
        data = json.load(f)

    pattern = pyembroidery.EmbPattern()
    for group_index, group in enumerate(data):
        thread = pyembroidery.EmbThread()
        thread.color = color_vec_to_hex_int(group["color"]["clr"])
        thread.description = f"Color {group["color"]["name"]}"
        thread.catalog_number = str(group_index)
        pattern.add_thread(thread)

        if group_index > 0:
            pattern.add_command(COLOR_CHANGE)

        for line in group["paths"]:
            sx, sy = mm_to_units(line["start"])

            # Move to the line start without stitching.
            pattern.add_stitch_absolute(JUMP, sx, sy)

            # Add actual stitch endpoints. If stitches include a third
            # element for editor-only stitch type, ignore it for VP3 output.
            for point_x, point_y, point_kind in line.get("stitches", []):
                x, y = mm_to_units((point_x, point_y))
                pattern.add_stitch_absolute(STITCH, x, y)

    pattern.add_command(END)
    pyembroidery.write_vp3(pattern, output_vp3_path)


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage:")
        print("  python convert.py input.json output.vp3")
        sys.exit(1)

    convert_json_to_vp3(sys.argv[1], sys.argv[2])
