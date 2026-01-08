import sys

import mido


def note_to_freq(note: int) -> int:
    return round(440 * (2 ** ((note - 69) / 12)))


def ticks_to_us(ticks, tempo, ticks_per_beat):
    return int(ticks * tempo / ticks_per_beat)


def convert(path: str):
    mid = mido.MidiFile(path)
    track = mido.merge_tracks(mid.tracks)

    tempo = 500000  # 120 BPM
    ticks_per_beat = mid.ticks_per_beat

    time_ticks = 0
    last_change_ticks = 0
    active_note = None

    for msg in track:
        time_ticks += msg.time

        if msg.type == "set_tempo":
            tempo = msg.tempo
            continue

        if msg.type == "note_on" and msg.velocity > 0:
            # End previous note if still active
            if active_note is not None:
                dt = time_ticks - last_change_ticks
                if dt > 0:
                    us = ticks_to_us(dt, tempo, ticks_per_beat)
                    freq = note_to_freq(active_note)
                    print(f"t{freq} {us}")
                last_change_ticks = time_ticks

            # Pause before note
            if active_note is None:
                dt = time_ticks - last_change_ticks
                if dt > 0:
                    us = ticks_to_us(dt, tempo, ticks_per_beat)
                    print(f"t0 {us}")
                    last_change_ticks = time_ticks

            active_note = msg.note

        elif msg.type in ("note_off", "note_on") and msg.velocity == 0:
            if active_note == msg.note:
                dt = time_ticks - last_change_ticks
                if dt > 0:
                    us = ticks_to_us(dt, tempo, ticks_per_beat)
                    freq = note_to_freq(active_note)
                    print(f"t{freq} {us}")
                active_note = None
                last_change_ticks = time_ticks


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: python midi_to_tones.py <file.mid>")
        sys.exit(1)

    convert(sys.argv[1])
