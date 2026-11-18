Mantel 0.5.4 WsQ, wersja 17. Start z Windows do zasobnika. Zawiera wszystko z wsq.16.

**Poprawione**
- **Start razem z Windows nie zasłania już ekranu.** Gdy Mantel uruchamia się przy starcie systemu, od razu chowa się do zasobnika obok zegara. Jeśli zasobnik jest wyłączony w ustawieniach, Mantel jest zminimalizowany. Pełny ekran nie pokazuje się nawet na chwilę, a zwiastuny nie odtwarzają się w tle.
- Gdy uruchomisz Mantel sam (skrót, menu Start), otwiera się jak dotąd na pełnym ekranie.

**Dobrze wiedzieć**
- Wpis autostartu z poprzednich wersji poprawia się sam przy pierwszym uruchomieniu wsq.17. Od następnego startu Windows Mantel startuje już w zasobniku.

**Pliki**
- `Mantel_0.5.4-wsq.17_x64-setup.exe`: instalator dla bieżącego użytkownika, bez uprawnień administratora.
- `Mantel_0.5.4-wsq.17_x64-portable.exe`: sam program, bez instalacji. Wymaga WebView2, który jest w Windows 11.

Pliki nie są podpisane cyfrowo, więc SmartScreen może pokazać ostrzeżenie. Wybierz wtedy *Więcej informacji → Uruchom mimo to*.

**VirusTotal (2026-11-18)**
- setup: 2/71 (APEX, Trapmine). To heurystyki ML, typowe dla niepodpisanego instalatora NSIS. [Raport](https://www.virustotal.com/gui/file/20c609d00a97469b44603b0980ba4ea6b2640ec37463edbff382fe63d0bcd337)
- portable: 1/71 (Trapmine, heurystyka ML). [Raport](https://www.virustotal.com/gui/file/92bbcd16db3eb464e8c810abd72aa62f6fb811800b82259cea3e0ea3e655917c)

**SHA-256**
```
20c609d00a97469b44603b0980ba4ea6b2640ec37463edbff382fe63d0bcd337  Mantel_0.5.4-wsq.17_x64-setup.exe
92bbcd16db3eb464e8c810abd72aa62f6fb811800b82259cea3e0ea3e655917c  Mantel_0.5.4-wsq.17_x64-portable.exe
```
