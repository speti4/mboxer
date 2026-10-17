# draft

ez egy greenfield project

## alaphelyzet

van egy ~13 GB méretű mbox fájlom, a céges levelezés korábban Gmailben ment, ezt takeouttal archiváltuk anno, ez lett az output:

"<archive path>"

a mappában mellette settings fájlok

ezt a teljes archive levelezést az alábbi opensource alkalmazással nyitottam meg eddig:

[eneam/mboxviewer](https://github.com/eneam/mboxviewer)

ez itt van a gépemen: "<local folder>"

## probléma

ez az mboxviewer app:
  1. nagyon rossz UI/UX-el rendelkezik
  2. instabil, lassú, gakran lefagy
  3. a keresés borzasztó felületen történik (Advanced Find), nincs rá hotkey, és nagyon kényelmetlen, nem felhasználóbarát

## igény

első körben keressünk ingyenes alternatívát, de nem hiszem hogy találunk megfelelőt

második körben fejlesszünk saját mbox viewer alkalmazás windows 11-re, ami a fenti problémákat megoldja
minél kevesebb (lehetőleg 0) dependency

## egyéb hasznos infok

csak 2-5 felhasználója lesz maximum az alkalmazásnak, main user én vagyok
heti max 1-2 alkalommal lesz szükségem a régi levelezésemből bármire, nem lesz heavy usage