Mantel 0.5.4 WsQ, wersja 20. Wymuszone wyłączanie przez shutdown.exe. Zawiera wszystko z wsq.19.

**Zmienione**
- **„Wyłącz” i „Uruchom ponownie” znowu korzystają z systemowego `shutdown.exe`, tym razem z wymuszeniem:** `shutdown /s /f /t 0` (restart: `/r /f /t 0`). Otwarte programy, także gry, zamykają się bez pytania o zapisanie zmian.
- Uruchamianie jest zabezpieczone:
  - Mantel bierze `shutdown.exe` wyłącznie z folderu System32, którego ścieżkę podaje sam Windows. Nie korzysta ze zmiennych środowiskowych i nie szuka pliku o tej nazwie w innych folderach.
  - Argumenty są stałe, polecenie idzie bez wiersza poleceń (`cmd`) i bez okna konsoli.
- Gdyby `shutdown.exe` się nie uruchomił, Mantel prosi o to samo wymuszone wyłączenie bezpośrednio Windows.

**Dobrze wiedzieć**
- Bitdefender zareagował wcześniej ochroną behawioralną (Atc4.Detection) na niepodpisany program w autostarcie wywołujący wyłączenie systemu. Jeśli zareaguje znowu, najpewniejszy jest wyjątek dla `mantel.exe` w *Ochrona → Advanced Threat Defense → Wyjątki*.

**Pliki**
- `Mantel_0.5.4-wsq.20_x64-setup.exe`: instalator dla bieżącego użytkownika, bez uprawnień administratora.
- `Mantel_0.5.4-wsq.20_x64-portable.exe`: sam program, bez instalacji. Wymaga WebView2, który jest w Windows 11.

Pliki nie są podpisane cyfrowo, więc SmartScreen może pokazać ostrzeżenie. Wybierz wtedy *Więcej informacji → Uruchom mimo to*.

**VirusTotal (2026-11-23)**
- setup: 2/70 (APEX, Trapmine). To heurystyki ML, typowe dla niepodpisanego instalatora NSIS. [Raport](https://www.virustotal.com/gui/file/ab1d7131a3b2b9e63a531638fffa9197f835ffbac920575524b50d35d419e842)
- portable: 1/70 (Trapmine, heurystyka ML). [Raport](https://www.virustotal.com/gui/file/84821d3ac9e6ccfed77de3be1bea7f462553059f422df928e2a0ea94f1097369)

**SHA-256**
```
ab1d7131a3b2b9e63a531638fffa9197f835ffbac920575524b50d35d419e842  Mantel_0.5.4-wsq.20_x64-setup.exe
84821d3ac9e6ccfed77de3be1bea7f462553059f422df928e2a0ea94f1097369  Mantel_0.5.4-wsq.20_x64-portable.exe
```
