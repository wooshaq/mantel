Mantel 0.5.4 WsQ, wersja 13. Poprawki danych w Statystykach. Zawiera wszystko z wsq.12.

**Poprawione**
- **Nazwy gier zamiast „steam:2072450”.** Dotyczy gier Steam, których nie masz już zainstalowanych. Nazwę i rodzaj aplikacji Mantel bierze z publicznej strony gry w sklepie Steam. Gdy sklep jej nie zna, pokazuje się „Gra Steam {numer}”.
- **Koniec z dniami po 24 godziny.**
  - Sesja dłuższa niż 12 godzin nie liczy się jako granie. Zwykle znaczy to, że Steam nie zapisał końca gry (był zamknięty albo komputer spał z otwartą grą) albo że program chodzi w tle.
  - Takie sesje znikają także z już zapisanej historii.
- **Same gry.** Programy ze Steama, które nie są grami (aplikacje, narzędzia, filmy), nie trafiają do statystyk.

**Pliki**
- `Mantel_0.5.4-wsq.13_x64-setup.exe`: instalator dla bieżącego użytkownika, bez uprawnień administratora.
- `Mantel_0.5.4-wsq.13_x64-portable.exe`: sam program, bez instalacji. Wymaga WebView2, który jest w Windows 11.

Pliki nie są podpisane cyfrowo, więc SmartScreen może pokazać ostrzeżenie. Wybierz wtedy *Więcej informacji → Uruchom mimo to*.

**VirusTotal (2026-11-17)**
- setup: 2/70 (APEX, Trapmine). To heurystyki ML, typowe dla niepodpisanego instalatora NSIS. [Raport](https://www.virustotal.com/gui/file/addf8237750ec3e7143e10227d6a9789ab90c1005e7c776d5479a14185ac514b)
- portable: 1/70 (Trapmine, heurystyka ML). [Raport](https://www.virustotal.com/gui/file/3dae8e0427bb38088cee60b280dfc0ea66b648a4574b3af29d0691f6810e3a18)

**SHA-256**
```
addf8237750ec3e7143e10227d6a9789ab90c1005e7c776d5479a14185ac514b  Mantel_0.5.4-wsq.13_x64-setup.exe
3dae8e0427bb38088cee60b280dfc0ea66b648a4574b3af29d0691f6810e3a18  Mantel_0.5.4-wsq.13_x64-portable.exe
```
