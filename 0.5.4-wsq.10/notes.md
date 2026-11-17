Mantel 0.5.4 WsQ, wersja 10. Poprawki do wsq.9.

**Poprawione**
- **Przyciski na stronie gry:** napis zaznaczonego przycisku pokazuje się w całości, np. „Osiągnięcia” zamiast uciętego „Osi”. Gdy przycisków było dużo, zaznaczony nie miał miejsca, żeby się rozwinąć.
- **Statystyki:** seria jednego dnia pokazuje się jako „1 dzień”, a nie „1 dni”.

**Pliki**
- `Mantel_0.5.4-wsq.10_x64-setup.exe`: instalator dla bieżącego użytkownika, bez uprawnień administratora.
- `Mantel_0.5.4-wsq.10_x64-portable.exe`: sam program, bez instalacji. Wymaga WebView2, który jest w Windows 11.

Pliki nie są podpisane cyfrowo, więc SmartScreen może pokazać ostrzeżenie. Wybierz wtedy *Więcej informacji → Uruchom mimo to*.

**VirusTotal (2026-11-17)**
- setup: 2/71 (APEX, Trapmine). To heurystyki ML, typowe dla niepodpisanego instalatora NSIS. [Raport](https://www.virustotal.com/gui/file/8e945f60d4ce1417facec3e278472572fd15f63df835cb124166623217564f2c)
- portable: 1/67 (Trapmine, heurystyka ML). [Raport](https://www.virustotal.com/gui/file/f89023f20dc68e78ae466dbeac84648bae7e352ef9572d3e2ee0e0a93e55e850)

**SHA-256**
```
8e945f60d4ce1417facec3e278472572fd15f63df835cb124166623217564f2c  Mantel_0.5.4-wsq.10_x64-setup.exe
f89023f20dc68e78ae466dbeac84648bae7e352ef9572d3e2ee0e0a93e55e850  Mantel_0.5.4-wsq.10_x64-portable.exe
```
