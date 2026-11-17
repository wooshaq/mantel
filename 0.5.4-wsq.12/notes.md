Mantel 0.5.4 WsQ, wersja 12. Dochodzi historia gry Steam wstecz i łatwiejsze łączenie z HowLongToBeat. Zawiera wszystko z wsq.11.

**Nowe**
- **Historia Steam wstecz.**
  - Mantel czyta dziennik, który klient Steam prowadzi o uruchamianych grach (`Steam\logs\gameprocess_log.txt` i jego starszą część). Każda sesja ma tam godzinę startu i końca.
  - Te sesje trafiają do kalendarza, wykresu dnia i historii gry, tak daleko wstecz, jak Steam trzyma te pliki.
  - Dzieje się to przy każdym odświeżeniu biblioteki, także przy starcie programu.
  - Sesje, które Mantel już zapisał sam, nie liczą się dwa razy.
  - Mantel tylko czyta te pliki. Niczego w nich nie zmienia.
- **HowLongToBeat w okienku.**
  - Przycisk „Wyszukaj na HowLongToBeat” otwiera stronę HLTB nad Mantelem, od razu z wynikami tej gry.
  - Klikasz właściwą grę, a Mantel sam bierze adres jej strony i zamyka okienko.
  - Wklejanie linku nadal działa.

**Warto wiedzieć**
- Steam z czasem nadpisuje najstarsze wpisy w dzienniku, więc zasięg wstecz zależy od tego, ile grasz. Od teraz Mantel przechowuje odczytane sesje u siebie, więc już nie przepadną.
- Historia wstecz obejmuje tylko gry Steam.

**Pliki**
- `Mantel_0.5.4-wsq.12_x64-setup.exe`: instalator dla bieżącego użytkownika, bez uprawnień administratora.
- `Mantel_0.5.4-wsq.12_x64-portable.exe`: sam program, bez instalacji. Wymaga WebView2, który jest w Windows 11.

Pliki nie są podpisane cyfrowo, więc SmartScreen może pokazać ostrzeżenie. Wybierz wtedy *Więcej informacji → Uruchom mimo to*.

**VirusTotal (2026-11-17)**
- setup: 2/71 (APEX, Trapmine). To heurystyki ML, typowe dla niepodpisanego instalatora NSIS. [Raport](https://www.virustotal.com/gui/file/c0e3daae87657c9abb9e48cecf93efbf054a2a1c15dbca718746d28fb6d3667b)
- portable: 1/71 (Trapmine, heurystyka ML). [Raport](https://www.virustotal.com/gui/file/031a9590e5d4bd831508945c7b7ac7efcb4d3e8076ef53c41973932db149098c)

**SHA-256**
```
c0e3daae87657c9abb9e48cecf93efbf054a2a1c15dbca718746d28fb6d3667b  Mantel_0.5.4-wsq.12_x64-setup.exe
031a9590e5d4bd831508945c7b7ac7efcb4d3e8076ef53c41973932db149098c  Mantel_0.5.4-wsq.12_x64-portable.exe
```
