#!/usr/bin/env python3
"""Parse one recorded egress-check destination table and refuse a bad claim.

Reads a transcript (argv 1) and writes the parsed rows (argv 2). Stdlib only.
Does not open a socket and does not print the transcript. A dashed separator
under the header is skipped, not counted.
"""

import sys

PERMITTED_REASONS = frozenset({"allowlisted", "loopback", "unix", "unspecified"})


def fail(reason):
    sys.stderr.write(reason + "\n")
    raise SystemExit(1)


def assignments(text, name):
    needle = name + "="
    found = []
    for line in text.splitlines():
        for token in line.split():
            if token.startswith(needle):
                found.append(token[len(needle) :])
    return found


def positive_int(values, label):
    if not values:
        fail(label + " is absent or not greater than zero")
    for value in values:
        try:
            number = int(value)
        except ValueError:
            fail(label + " is absent or not greater than zero")
        if number <= 0:
            fail(label + " is absent or not greater than zero")
    return values[0]


def parse_table(text):
    lines = text.splitlines()
    header_at = None
    for index, line in enumerate(lines):
        if line.startswith("count") and "destination" in line.split():
            header_at = index
            break
    if header_at is None:
        fail("no observed-destinations table found")

    rows = []
    for line in lines[header_at + 1 :]:
        if not line.strip():
            break
        parts = line.split()
        if parts and parts[0] and set(parts[0]) <= {"-"}:
            continue
        if len(parts) != 7:
            fail("destination row does not have 7 columns")
        rows.append(parts)
    if not rows:
        fail("no observed-destinations table found")
    return rows


def main(argv):
    if len(argv) != 3:
        fail("usage: destination-check.py TRANSCRIPT DESTINATIONS")
    try:
        text = open(argv[1], encoding="utf-8", errors="replace").read()
    except OSError as exc:
        fail("cannot read transcript: " + str(exc))

    rows = parse_table(text)
    for _count, _family, _destination, _port, _host, status, reason in rows:
        if status != "PASS":
            fail("row status is not PASS: " + status)
        if reason not in PERMITTED_REASONS:
            fail("row reason outside permitted set: " + reason)

    violations = assignments(text, "EGRESS_VIOLATIONS")
    if not violations or any(value != "0" for value in violations):
        fail("EGRESS_VIOLATIONS= is absent or not 0")
    api = positive_int(assignments(text, "API_CONNECTS"), "API_CONNECTS=")
    web = positive_int(assignments(text, "WEB_FETCH_CONNECTS"), "WEB_FETCH_CONNECTS=")

    try:
        with open(argv[2], "w", encoding="utf-8") as handle:
            for count, family, destination, port, host, status, reason in rows:
                handle.write(
                    "%s %s %s %s %s %s %s\n"
                    % (count, family, destination, port, host, status, reason)
                )
    except OSError as exc:
        fail("cannot write destinations: " + str(exc))

    sys.stdout.write(
        "parsed %d rows; API_CONNECTS=%s; WEB_FETCH_CONNECTS=%s\n" % (len(rows), api, web)
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
