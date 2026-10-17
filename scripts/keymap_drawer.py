#!/usr/bin/env python3
"""Convert keyboard.toml [keymap] into keymap-drawer input (YAML via JSON) and a QMK-style layout JSON.

Usage: python3 scripts/keymap_drawer.py keyboard.toml keymap-drawer/madula.yaml keymap-drawer/madula_layout.json
Then:  keymap draw -j keymap-drawer/madula_layout.json keymap-drawer/madula.yaml > keymap-drawer/madula.svg

Only the physical keys are drawn: Cornix rows 0-3 (with both encoder pushes) and the three Madula keys.
The IQS virtual gesture row is skipped.
"""
import json
import re
import sys
import tomllib

MOD = {"LCtrl": "⌃", "LShift": "⇧", "LAlt": "⌥", "LGui": "⌘", "RCtrl": "⌃", "RShift": "⇧", "RAlt": "⌥", "RGui": "⌘"}
NAME = {"Escape": "Esc", "Backspace": "⌫", "Enter": "⏎", "Space": "␣", "Tab": "⇥", "Delete": "⌦",
        "Language1": "かな", "Language2": "英数", "Semicolon": ";", "Quote": "'", "Minus": "-", "Equal": "=",
        "Comma": ",", "Dot": ".", "Slash": "/", "Backslash": "\\", "Grave": "`", "LeftBracket": "[",
        "RightBracket": "]", "Left": "←", "Right": "→", "Up": "↑", "Down": "↓", "PageUp": "PgUp",
        "PageDown": "PgDn", "KpEqual": "Kp=", "AudioMute": "Mute", "AudioVolUp": "Vol+", "AudioVolDown": "Vol-",
        "MouseBtn1": "LClick", "MouseBtn2": "RClick", "MouseBtn3": "MClick", "MouseWheelUp": "Wh↑",
        "MouseWheelDown": "Wh↓", "User22": "Dial", "No": ""}


def tokens(line):
    out, depth, cur = [], 0, ""
    for ch in line:
        if ch == " " and depth == 0:
            if cur:
                out.append(cur)
            cur = ""
            continue
        depth += ch == "("
        depth -= ch == ")"
        cur += ch
    if cur:
        out.append(cur)
    return out


def simple(t):
    if t in NAME:
        return NAME[t]
    if t in MOD:
        return MOD[t]
    if re.fullmatch(r"Kc\d", t):
        return t[2:]
    m = re.fullmatch(r"WM\((\w+), (.+)\)", t)
    if m:
        return "".join(MOD[x.strip()] for x in m.group(2).split("|")) + simple(m.group(1))
    m = re.fullmatch(r"MO\((\d+)\)", t)
    if m:
        return f"L{m.group(1)}"
    return t


def key(t, morses):
    if t == "_":
        return {"type": "trans"}
    m = re.fullmatch(r"LT\((\d+), (.+?)(?:, \w+)?\)", t)
    if m:
        return {"t": simple(m.group(2)), "h": f"L{m.group(1)}"}
    m = re.fullmatch(r"MT\((.+?), (\w+)(?:, \w+)?\)", t)
    if m:
        return {"t": simple(m.group(1)), "h": simple(m.group(2))}
    m = re.fullmatch(r"TD\((\d+)\)", t)
    if m:
        d = morses[int(m.group(1))]
        out = {"t": simple(d.get("tap", "No"))}
        if "hold" in d:
            out["h"] = simple(d["hold"])
        if "double_tap" in d:
            out["s"] = "×2 " + simple(d["double_tap"])
        return out
    return simple(t)


def main(src, out_yaml, out_layout):
    cfg = tomllib.load(open(src, "rb"))
    morses = cfg.get("behavior", {}).get("morse", {}).get("morses", [])
    layers = {}
    for i, layer in enumerate(cfg["keymap"]["layer"]):
        rows = [tokens(r) for r in layer["keys"].strip().split("\n")]
        flat = [t for r in rows[:5] for t in r]  # Cornix rows 0-3 + Madula keys; skip IQS row
        layers[f"L{i}"] = [key(t, morses) for t in flat]
    pos = []
    for r in range(4):
        pos += [{"x": c, "y": r} for c in range(6)]
        if r == 2:
            pos += [{"x": 6.5, "y": 2}, {"x": 7.5, "y": 2}]
        pos += [{"x": 9 + c, "y": r} for c in range(6)]
    pos += [{"x": 9.5 + c, "y": 4.5} for c in range(3)]
    for name, keys in layers.items():
        assert len(keys) == len(pos), (name, len(keys), len(pos))
    json.dump({"layout": {"qmk_info_json": out_layout}, "layers": layers}, open(out_yaml, "w"), ensure_ascii=False, indent=1)
    json.dump({"keyboard_name": "Cornix + Madula (RMK)", "layouts": {"LAYOUT": {"layout": pos}}}, open(out_layout, "w"), indent=1)


if __name__ == "__main__":
    main(*sys.argv[1:])
