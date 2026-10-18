"""Writes target/demo/demo.mbox: a made-up Gmail Takeout archive for screenshots (task 026).

Every person, address and message is invented; the addresses sit under example.com / .org / .net.
The file has what the viewer shows off: Gmail labels with sub-labels and categories (X-Gmail-Labels),
conversations (X-GM-THRID), an HTML newsletter with a remote image, attachments, and Hungarian
accents for the accent-folding search. Deterministic: the same bytes on every run. Standard library
only, not part of the build; target/ is gitignored, so the mbox never enters the repo.

    python -I scripts/make-demo-mbox.py              # writes target/demo/demo.mbox
    python -I scripts/make-demo-mbox.py --out X.mbox
"""

from __future__ import annotations

import argparse
import struct
import zlib
from datetime import datetime, timedelta, timezone
from email.headerregistry import Address
from email.message import EmailMessage
from email.policy import default
from email.utils import format_datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
POLICY = default.clone(linesep="\n")
TZ = timezone(timedelta(hours=2))
START = datetime(2024, 11, 4, 9, 0, tzinfo=TZ)

ME = ("Alex Morgan", "alex@example.com")
PEOPLE = {
    "dora": ("Dóra Kovács", "dora.kovacs@example.org"),
    "sam": ("Sam Patel", "sam@northwind.example.com"),
    "lena": ("Lena Fischer", "lena.fischer@example.net"),
    "mark": ("Mark Ellis", "billing@studio-ellis.example.com"),
    "rita": ("Rita Szabó", "rita@example.org"),
    "tom": ("Tom Reyes", "tom.reyes@example.net"),
    "air": ("SkyLine Bookings", "noreply@skyline.example.com"),
    "hotel": ("Hotel Riverside", "reservations@riverside.example.net"),
    "news": ("The Weekly Byte", "newsletter@weeklybyte.example.org"),
    "bank": ("Example Bank", "statements@bank.example.com"),
    "git": ("CodeHost", "notifications@codehost.example.com"),
}

# Each thread: (thread id, labels on received messages, subject, [(day offset, hour, sender key or "me", body, attachments)]).
# Sent messages carry "Sent" instead of the thread's received labels.
THREADS = [
    ("1801", "Inbox,Important,Work/Clients", "Website relaunch: timeline and budget", [
        (0, 9, "sam", "Hi Alex,\n\nwe would like to relaunch the Northwind website before the spring campaign.\n"
                      "Could you send a rough timeline and a budget range by Friday?\n\nThanks,\nSam", []),
        (1, 14, "me", "Hi Sam,\n\nhappy to. A first draft: two weeks discovery, four weeks design and build,\n"
                      "one week testing. I will attach the estimate tomorrow.\n\nAlex", []),
        (2, 10, "me", "Sam,\n\nthe estimate is attached. The range covers the content migration as well.\n\nAlex",
         [("northwind-estimate.pdf", "pdf")]),
        (4, 16, "sam", "Looks good. Let us start on the 18th. I will set up the kick-off call.\n\nSam", []),
    ]),
    ("1802", "Inbox,Work/Invoices", "Számla — October design work", [
        (6, 8, "mark", "Hello Alex,\n\nplease find attached the invoice for the October design work.\n"
                       "Payment is due within 14 days.\n\nBest regards,\nMark Ellis", [("invoice-2024-10.pdf", "pdf")]),
        (7, 11, "me", "Thanks Mark, paid today.\n\nAlex", []),
    ]),
    ("1803", "Inbox,Travel", "Your flight to Lisbon is confirmed", [
        (9, 7, "air", "Booking reference: QX7K2P\n\nBudapest (BUD) → Lisbon (LIS)\nDeparture 06:40, arrival 09:15 local time\n"
                      "Seat 14C, one cabin bag.\n\nHave a pleasant trip!", [("boarding-pass.png", "png")]),
    ]),
    ("1804", "Inbox,Travel", "Reservation at Hotel Riverside", [
        (10, 12, "hotel", "Dear Mr Morgan,\n\nwe are pleased to confirm your stay from 14 to 18 March,\n"
                          "a double room with a river view, breakfast included.\n\nKind regards,\nHotel Riverside",
         [("reservation.ics", "ics")]),
        (11, 9, "me", "Thank you. Could we check in early, around 11:00?\n\nAlex Morgan", []),
        (11, 15, "hotel", "Of course, the room will be ready at 11:00. We look forward to welcoming you.", []),
    ]),
    ("1805", "Inbox,Family", "Hétvégi ebéd vasárnap?", [
        (13, 18, "dora", "Szia Alex!\n\nVasárnap jöttök ebédre? Rakott krumpli lesz és meggyes pite.\n\nPuszi,\nDóra", []),
        (13, 20, "me", "Szia! Persze, egyre ott vagyunk. Vigyünk valamit?\n\nAlex", []),
        (14, 8, "dora", "Csak jó kedvet. :)", []),
    ]),
    ("1806", "Inbox,Category Updates", "Monthly statement for October", [
        (15, 6, "bank", "Your statement for October is ready. Sign in to your online banking to view it.\n\n"
                        "This is an automated message, please do not reply.", []),
    ]),
    ("1807", "Inbox,Work/Clients", "Photos for the About page", [
        (17, 10, "lena", "Hi Alex,\n\nhere are the team photos from Tuesday. Three of them should work for the\n"
                         "About page; the full set is on the shared drive.\n\nLena", [("team-photo.png", "png")]),
        (17, 13, "me", "Great shots, thank you! I will use the second one for the header.\n\nAlex", []),
    ]),
    ("1808", "Inbox,Category Updates", "[northwind/site] Pull request #42 merged", [
        (19, 16, "git", "Merged #42 into main: navigation rewritten with accessible menus.\n\n3 files changed, "
                        "142 insertions, 87 deletions.", []),
    ]),
    ("1809", "Inbox,Important", "Contract renewal for next year", [
        (21, 9, "tom", "Hi Alex,\n\nour support contract ends in December. Shall we renew it on the same terms?\n"
                       "I can send the paperwork this week.\n\nTom", []),
        (22, 11, "me", "Hi Tom, yes, the same terms work for us. Please send it over.\n\nAlex", []),
        (24, 15, "tom", "Sent for signature. Thanks for another good year!\n\nTom", [("support-contract-2025.pdf", "pdf")]),
    ]),
    ("1810", "Inbox,Work/Invoices", "Invoice 2024-11 from Studio Ellis", [
        (36, 8, "mark", "Hello Alex,\n\nthe invoice for November is attached.\n\nBest regards,\nMark", [("invoice-2024-11.pdf", "pdf")]),
    ]),
    ("1811", "Inbox,Family", "Rita születésnapja", [
        (40, 19, "rita", "Szia!\n\nSzombaton este hétkor ünnepelünk a kertben. Ha esik, akkor bent.\n"
                         "Hozzatok jó kedvet és pulóvert!\n\nRita", []),
        (41, 9, "me", "Ott leszünk! Mit szólnál egy közös ajándékhoz Dórával?", []),
        (41, 12, "rita", "Az nagyon kedves, de tényleg semmi nem kell. :)", []),
    ]),
    ("1812", "Inbox,Work/Clients", "Kick-off notes and next steps", [
        (45, 17, "sam", "Hi Alex,\n\nthanks for the kick-off today. My notes:\n\n- sitemap draft by next Wednesday\n"
                        "- content owners named per section\n- weekly check-in on Thursdays at 10:00\n\nSam", []),
        (46, 9, "me", "Thanks Sam, all agreed. The sitemap draft is on its way.\n\nAlex", []),
    ]),
    ("1813", "Inbox,Travel", "Lisbon: restaurant tips", [
        (52, 20, "lena", "You are going to Lisbon? Try the little place behind the tram stop in Alfama,\n"
                         "and the custard tarts in Belém are worth the queue.\n\nLena", []),
        (53, 8, "me", "Noted, thank you! I will bring you a box of tarts.\n\nAlex", []),
    ]),
    ("1814", "Inbox,Work/Invoices", "Invoice 2024-12 from Studio Ellis", [
        (66, 8, "mark", "Hello Alex,\n\nthe invoice for December is attached. Happy holidays!\n\nMark", [("invoice-2024-12.pdf", "pdf")]),
        (67, 10, "me", "Thanks Mark, happy holidays to you too.\n\nAlex", []),
    ]),
    ("1815", "Inbox,Important,Work/Clients", "Go-live checklist", [
        (300, 9, "sam", "Hi Alex,\n\nwe are close. Before go-live I would like to confirm redirects, analytics,\n"
                        "the cookie banner and the 404 page. Can we walk through them on Thursday?\n\nSam", []),
        (300, 15, "me", "Hi Sam,\n\nyes. Redirects and the 404 page are done; analytics goes in tomorrow.\n"
                        "The checklist is attached with the open items marked.\n\nAlex", [("go-live-checklist.pdf", "pdf")]),
        (301, 10, "sam", "Perfect, Thursday at 10:00 then.", []),
        (303, 12, "me", "Everything is green. We can switch the DNS on Monday morning.\n\nAlex", []),
        (304, 9, "sam", "Wonderful news. Thank you and the team for the great work!\n\nSam", []),
    ]),
    ("1816", "Inbox,Category Updates", "[northwind/site] Release v2.0.0 published", [
        (306, 11, "git", "Release v2.0.0 of northwind/site is out.\n\nHighlights: the new design, faster pages,\n"
                         "accessible navigation, and a search that finds 'kávé' when you type 'kave'.", []),
    ]),
    ("1817", "Inbox,Family", "Fotók a nyaralásról", [
        (320, 21, "dora", "Szia!\n\nKüldöm a balatoni képeket. A naplementés a kedvencem.\n\nPuszi,\nDóra",
         [("balaton-sunset.png", "png")]),
        (321, 8, "me", "Gyönyörűek! A naplementés megy a falra.\n\nAlex", []),
    ]),
    ("1818", "Inbox,Work/Invoices", "Invoice 2025-09 from Studio Ellis", [
        (330, 8, "mark", "Hello Alex,\n\nthe invoice for September is attached.\n\nBest regards,\nMark", [("invoice-2025-09.pdf", "pdf")]),
    ]),
    ("1819", "Inbox,Important", "Quarterly planning workshop", [
        (335, 10, "tom", "Hi Alex,\n\ncould you join our quarterly planning workshop on the 12th? We would love\n"
                         "an outside view on the roadmap. Half a day, lunch included.\n\nTom", []),
        (335, 16, "me", "Hi Tom, with pleasure. Should I prepare anything?\n\nAlex", []),
        (336, 9, "tom", "Just bring your questions. The agenda follows next week.\n\nTom", []),
    ]),
    ("1820", "Inbox,Work/Clients", "Analytics after the first month", [
        (340, 9, "sam", "Hi Alex,\n\nthe first month after the relaunch: bounce rate down by a third, and the\n"
                        "contact form brings twice as many requests. Thanks again!\n\nSam", []),
        (340, 14, "me", "That is great to hear, Sam. Shall we plan the next round of improvements?\n\nAlex", []),
        (341, 10, "sam", "Yes, let us talk on Thursday. I will bring the wish list.\n\nSam", []),
    ]),
]

NEWSLETTER_DAYS = [8, 38, 68, 310, 338]
NEWSLETTER_HTML = """<html><body style="font-family:Segoe UI,Arial,sans-serif;color:#1f2328;margin:0">
<table width="100%" cellpadding="0" cellspacing="0"><tr><td style="background:#1f5ea0;padding:18px 24px">
<img src="https://images.weeklybyte.example.org/logo.png" alt="The Weekly Byte" width="160" height="32">
</td></tr><tr><td style="padding:20px 24px">
<h1 style="font-size:22px;margin:0 0 12px">Issue {issue}: {title}</h1>
<p style="font-size:15px;line-height:1.5">{lead}</p>
<ul style="font-size:15px;line-height:1.6">
<li><a href="https://weeklybyte.example.org/{issue}/one">{item1}</a></li>
<li><a href="https://weeklybyte.example.org/{issue}/two">{item2}</a></li>
<li><a href="https://weeklybyte.example.org/{issue}/three">{item3}</a></li>
</ul>
<p style="font-size:13px;color:#59636e">You receive this because you subscribed at weeklybyte.example.org.</p>
</td></tr></table></body></html>
"""
NEWSLETTER_ISSUES = [
    ("Fast search on a laptop", "Full-text indexes are not just for servers.",
     "How an inverted index works", "Accent folding explained", "Memory-mapped files in practice"),
    ("Small tools, long lives", "Single-file programs age well.",
     "Why portable apps still matter", "Bundling a web UI into one exe", "A short history of mbox"),
    ("Year in review", "The twelve stories you read most this year.",
     "Top 3: the parser that never stops", "Top 2: sandboxing HTML mail", "Top 1: search in under a second"),
    ("Back from the summer break", "New series: building desktop apps with web views.",
     "Part 1: the shell", "Part 2: the IPC bridge", "Part 3: the content security policy"),
    ("Archives you can trust", "Keeping old mail readable for decades.",
     "Open formats win", "Indexes are caches", "Never modify the original"),
]


def png_bytes(width: int, height: int, rgb: tuple[int, int, int]) -> bytes:
    """A flat-colored PNG, built by hand so the script needs nothing beyond the standard library."""
    def chunk(kind: bytes, data: bytes) -> bytes:
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)

    row = b"\x00" + bytes(rgb) * width
    raw = zlib.compress(row * height, 9)
    header = struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header) + chunk(b"IDAT", raw) + chunk(b"IEND", b"")


def pdf_bytes(title: str) -> bytes:
    """A one-page PDF with the title as its only text."""
    text = title.encode("latin-1", "replace")
    stream = b"BT /F1 18 Tf 72 720 Td (" + text + b") Tj ET"
    objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>",
        b"<< /Length " + str(len(stream)).encode() + b" >>\nstream\n" + stream + b"\nendstream",
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
    ]
    out = b"%PDF-1.4\n"
    offsets = []
    for number, body in enumerate(objects, start=1):
        offsets.append(len(out))
        out += f"{number} 0 obj\n".encode() + body + b"\nendobj\n"
    xref = len(out)
    out += f"xref\n0 {len(objects) + 1}\n0000000000 65535 f \n".encode()
    out += b"".join(f"{offset:010d} 00000 n \n".encode() for offset in offsets)
    out += f"trailer\n<< /Size {len(objects) + 1} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n".encode()
    return out


def ics_bytes() -> bytes:
    return (
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Hotel Riverside//EN\r\nBEGIN:VEVENT\r\n"
        "UID:demo-reservation@riverside.example.net\r\nDTSTAMP:20241114T120000Z\r\n"
        "DTSTART;VALUE=DATE:20250314\r\nDTEND;VALUE=DATE:20250318\r\nSUMMARY:Hotel Riverside\r\n"
        "END:VEVENT\r\nEND:VCALENDAR\r\n"
    ).encode()


def attachment(name: str, kind: str) -> tuple[bytes, str, str]:
    if kind == "pdf":
        return pdf_bytes(name.removesuffix(".pdf").replace("-", " ").title()), "application", "pdf"
    if kind == "png":
        colour = (31, 94, 160) if "photo" in name else (230, 140, 60)
        return png_bytes(64, 40, colour), "image", "png"
    return ics_bytes(), "text", "calendar"


def address(person: tuple[str, str]) -> str:
    name, email = person
    user, domain = email.split("@")
    return str(Address(name, user, domain))


def message(when: datetime, sender: tuple[str, str], to: tuple[str, str], subject: str, body: str,
            message_id: str, reply_to: str | None, files: list[tuple[str, str]], html: str | None = None) -> EmailMessage:
    msg = EmailMessage(policy=POLICY)
    msg["From"] = address(sender)
    msg["To"] = address(to)
    msg["Subject"] = subject
    msg["Date"] = format_datetime(when)
    msg["Message-ID"] = message_id
    if reply_to:
        msg["In-Reply-To"] = reply_to
        msg["References"] = reply_to
    msg.set_content(body)
    if html:
        msg.add_alternative(html, subtype="html")
    for name, kind in files:
        data, maintype, subtype = attachment(name, kind)
        msg.add_attachment(data, maintype=maintype, subtype=subtype, filename=name)
    # Fixed boundaries: the email package picks random ones, and the output must not change between runs.
    for number, part in enumerate(msg.walk()):
        if part.is_multipart():
            part.set_boundary(f"demo-{message_id.strip('<>').split('@')[0]}-{number}")
    return msg


def mbox_entry(thread: str, labels: str, when: datetime, msg: EmailMessage) -> str:
    stamp = when.astimezone(timezone.utc).strftime("%a %b %d %H:%M:%S +0000 %Y")
    head = f"From {thread}@xxx {stamp}\nX-GM-THRID: {thread}\nX-Gmail-Labels: {labels}\n"
    text = msg.as_string(policy=POLICY)
    # mboxrd: a body line starting with "From " (after any ">") gains one more ">".
    lines = [(">" + line) if line.lstrip(">").startswith("From ") else line for line in text.split("\n")]
    return head + "\n".join(lines).rstrip("\n") + "\n\n"


def build() -> str:
    entries: list[tuple[datetime, str]] = []
    for thread, labels, subject, posts in THREADS:
        previous = None
        first_sender = next(PEOPLE[who] for _, _, who, _, _ in posts if who != "me")
        for index, (day, hour, who, body, files) in enumerate(posts):
            when = START + timedelta(days=day, hours=hour - START.hour)
            mine = who == "me"
            sender, to = (ME, first_sender) if mine else (PEOPLE[who], ME)
            msg_id = f"<t{thread}m{index}@mail.example.com>"
            title = subject if index == 0 else f"Re: {subject}"
            msg = message(when, sender, to, title, body, msg_id, previous, files)
            entry_labels = "Sent" if mine else (labels if index == 0 else labels.replace(",Important", ""))
            entries.append((when, mbox_entry(thread, entry_labels + ",Opened", when, msg)))
            previous = msg_id
    for number, (day, issue) in enumerate(zip(NEWSLETTER_DAYS, NEWSLETTER_ISSUES), start=1):
        when = START + timedelta(days=day, hours=-2)
        title, lead, *items = issue
        html = NEWSLETTER_HTML.format(issue=number, title=title, lead=lead, item1=items[0], item2=items[1], item3=items[2])
        plain = f"Issue {number}: {title}\n\n{lead}\n\n" + "\n".join(f"- {item}" for item in items)
        msg_id = f"<news{number}@weeklybyte.example.org>"
        msg = message(when, PEOPLE["news"], ME, f"The Weekly Byte #{number}: {title}", plain, msg_id, None, [], html)
        entries.append((when, mbox_entry(f"19{number:02d}", "Inbox,Category Promotions,Opened", when, msg)))
    entries.sort(key=lambda item: item[0])
    return "".join(text for _, text in entries)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--out", type=Path, default=ROOT / "target" / "demo" / "demo.mbox")
    args = parser.parse_args()
    text = build()
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_bytes(text.encode("utf-8"))
    print(f"{args.out}: {text.count(chr(10) + 'From ') + 1} messages, {len(text.encode('utf-8'))} bytes")


if __name__ == "__main__":
    main()
