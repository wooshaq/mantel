Mantel 0.5.4 WsQ, wersja 16. Szybszy powrót z gry. Zawiera wszystko z wsq.15.

**Lepsze**
- **Mantel od razu wie, że gra się skończyła.** Dotąd sprawdzał procesy co 3 sekundy i czekał na trzy takie sprawdzenia, więc powrót trwał do 9 sekund. Teraz Windows sam powiadamia Mantel w chwili zamknięcia gry, a Mantel czeka już tylko sekundę, na wypadek gdyby gra uruchamiała się ponownie (np. przez launcher albo anti-cheat).
- **Czas gry liczy się do chwili zamknięcia gry**, a nie do następnego sprawdzenia.

**Poprawione**
- **Big Picture po grze** (dla włączonej opcji „Big Picture” w Mantelu). Steam zwykle ignoruje pierwsze polecenie zamknięcia BP, bo jeszcze kończy grę, np. zapisuje dane w chmurze. Mantel ponawia je teraz przez kilka sekund i dopiero potem wraca na wierzch, także wtedy, gdy Windows nie chce przepuścić programu z tła.

**Dobrze wiedzieć**
- Jeśli masz w Steamie włączone *Ustawienia → W grze → Używaj nakładki Big Picture, gdy korzystasz z kontrolera*, opcja „Big Picture” w Mantelu nie jest potrzebna. Nakładka BP i tak działa w każdej grze.

**Pliki**
- `Mantel_0.5.4-wsq.16_x64-setup.exe`: instalator dla bieżącego użytkownika, bez uprawnień administratora.
- `Mantel_0.5.4-wsq.16_x64-portable.exe`: sam program, bez instalacji. Wymaga WebView2, który jest w Windows 11.

Pliki nie są podpisane cyfrowo, więc SmartScreen może pokazać ostrzeżenie. Wybierz wtedy *Więcej informacji → Uruchom mimo to*.

**VirusTotal (2026-11-18)**
- setup: 2/71 (APEX, Trapmine). To heurystyki ML, typowe dla niepodpisanego instalatora NSIS. [Raport](https://www.virustotal.com/gui/file/38c737d79a646af3d141901a8ad4c47c32b9becbe8caccc3e35895212b012cf5)
- portable: 1/71 (Trapmine, heurystyka ML). [Raport](https://www.virustotal.com/gui/file/a6bb47eb54e4ea59d144df7a7070d1ff11f5b8f41ef0ff688372c7ddce76b430)

**SHA-256**
```
38c737d79a646af3d141901a8ad4c47c32b9becbe8caccc3e35895212b012cf5  Mantel_0.5.4-wsq.16_x64-setup.exe
a6bb47eb54e4ea59d144df7a7070d1ff11f5b8f41ef0ff688372c7ddce76b430  Mantel_0.5.4-wsq.16_x64-portable.exe
```
