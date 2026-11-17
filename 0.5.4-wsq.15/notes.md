Mantel 0.5.4 WsQ, wersja 15. RTX HDR dla wybranych gier. Zawiera wszystko z wsq.14.

**Nowe**
- **RTX HDR dla wybranych gier.** Przy metodzie HDR „NVIDIA” (*Ustawienia → System*) przycisk HDR na stronie gry przełącza się kolejno: wyłączone → HDR → RTX HDR. Przy RTX HDR sterownik NVIDIA zamienia obraz SDR gry w HDR.
  - Mantel ustawia RTX HDR w profilu sterownika tylko dla tej gry i tylko na czas gry, a po jej zamknięciu przywraca profil do poprzedniego stanu. Jeśli Mantel albo komputer padnie w trakcie gry, przy następnym starcie Mantel sam to odkręci.
  - Ekran przełącza się przy tym w HDR, bo RTX HDR potrzebuje ekranu w trybie HDR.
  - Nie trzeba NVIDIA App ani jej nakładki. Wymagana karta RTX.

**Dobrze wiedzieć**
- RTX HDR jest dla gier bez własnego HDR. W grach, które mają własne HDR, wybierz zwykłe „HDR”.
- RTX HDR kosztuje trochę wydajności, zwykle kilka procent klatek.
- Tego ustawienia sterownika NVIDIA oficjalnie nie dokumentuje (nazwę i numer zna NVIDIA Profile Inspector). Jeśli przyszły sterownik je zmieni, RTX HDR po prostu się nie włączy, a gra uruchomi się normalnie.

**Pliki**
- `Mantel_0.5.4-wsq.15_x64-setup.exe`: instalator dla bieżącego użytkownika, bez uprawnień administratora.
- `Mantel_0.5.4-wsq.15_x64-portable.exe`: sam program, bez instalacji. Wymaga WebView2, który jest w Windows 11.

Pliki nie są podpisane cyfrowo, więc SmartScreen może pokazać ostrzeżenie. Wybierz wtedy *Więcej informacji → Uruchom mimo to*.

**VirusTotal (2026-11-17)**
- setup: 2/69 (APEX, Trapmine). To heurystyki ML, typowe dla niepodpisanego instalatora NSIS. [Raport](https://www.virustotal.com/gui/file/815cb2879d8805be4be625df387a6d0b8448bfba1943862f3061d4c09febeb99)
- portable: 1/65 (Trapmine, heurystyka ML). [Raport](https://www.virustotal.com/gui/file/789097e1e76735cdf9080c7befb0ea19f33f584a79d41a6d022886eaadf8e160)

**SHA-256**
```
815cb2879d8805be4be625df387a6d0b8448bfba1943862f3061d4c09febeb99  Mantel_0.5.4-wsq.15_x64-setup.exe
789097e1e76735cdf9080c7befb0ea19f33f584a79d41a6d022886eaadf8e160  Mantel_0.5.4-wsq.15_x64-portable.exe
```
