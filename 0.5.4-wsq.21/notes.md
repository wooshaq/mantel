Mantel 0.5.4 WsQ, wersja 21. Własne parametry wyłączania i restartu. Zawiera wszystko z wsq.20.

**Nowe**
- **Parametry wyłączania i ponownego uruchomienia** w *Ustawienia → System*. Wpisujesz, co Mantel ma przekazać do `shutdown.exe`. Puste pole oznacza domyślne `/s /f /t 0` (wyłączanie) i `/r /f /t 0` (restart).
  - Przykłady: `/s /hybrid /f /t 0` (szybki rozruch), `/s /t 30` (za 30 sekund, bez wymuszania), `/h` (hibernacja).
  - Na klawiaturze ekranowej nie ma ukośnika, ale `shutdown.exe` przyjmuje też myślnik: `-s -f -t 0` działa tak samo.

**Bezpieczeństwo**
- Mantel przepuszcza tylko parametry `shutdown.exe` dotyczące tego komputera: `/s /r /g /sg /h /p /l /a /hybrid /fw /o /soft /f`, `/t` z liczbą sekund i `/d` z kodem powodu.
- Odrzuca `/m` (inny komputer w sieci), `/c` (dowolny tekst), `/i` i `/e`, a także wszystko, co nie jest parametrem, np. `& calc`.
- Każdy parametr trafia do `shutdown.exe` osobno, bez wiersza poleceń (`cmd`). Sprawdzanie odbywa się przy zapisie i ponownie tuż przed użyciem. Gdyby coś nie przeszło, Mantel użyje parametrów domyślnych.

**Pliki**
- `Mantel_0.5.4-wsq.21_x64-setup.exe`: instalator dla bieżącego użytkownika, bez uprawnień administratora.
- `Mantel_0.5.4-wsq.21_x64-portable.exe`: sam program, bez instalacji. Wymaga WebView2, który jest w Windows 11.

Pliki nie są podpisane cyfrowo, więc SmartScreen może pokazać ostrzeżenie. Wybierz wtedy *Więcej informacji → Uruchom mimo to*.

**VirusTotal (2026-11-23)**
- setup: 2/70 (APEX, Trapmine). To heurystyki ML, typowe dla niepodpisanego instalatora NSIS. [Raport](https://www.virustotal.com/gui/file/3384377367a77d4d797a9a18d0c8bdca8b63334107134d8a38de87524f73ec80)
- portable: 1/70 (Trapmine, heurystyka ML). [Raport](https://www.virustotal.com/gui/file/0c76ac042727787641bc714959dd34676adfaa7d0bcc047b060b5dcf4288afa4)

**SHA-256**
```
3384377367a77d4d797a9a18d0c8bdca8b63334107134d8a38de87524f73ec80  Mantel_0.5.4-wsq.21_x64-setup.exe
0c76ac042727787641bc714959dd34676adfaa7d0bcc047b060b5dcf4288afa4  Mantel_0.5.4-wsq.21_x64-portable.exe
```
