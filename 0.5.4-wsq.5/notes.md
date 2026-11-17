Mantel 0.5.4 WsQ, wersja 5: **bez żadnego logowania**. Zawiera wszystko z wsq.4: tray, osiągnięcia Steam z publicznego profilu, Big Picture i HDR na czas gry.

**Usunięte**
- **Logowanie do Steama, Xboxa i Microsoftu.** W programie nie ma już okien logowania, tokenów ani kodu szyfrującego tokeny. Mantel nie łączy się z żadnym kontem.
- Strona **Konta** w ustawieniach i przyciski „Połącz” w konfiguracji.
- Kafelki „Zaloguj się” w Bibliotece. Biblioteka pokazuje gry zainstalowane na tym komputerze.

**Co zostaje**
- **Osiągnięcia Steam z publicznego profilu, bez logowania.** Ustawienia → Biblioteka → Osiągnięcia Steam.
- **Czas gry.** Mantel liczy go sam, także dla gier uruchomionych poza nim, a dla gier Steam czyta go z plików Steama.

**Warto wiedzieć**
- Jeśli starsza wersja zapisała jakieś logowanie, przy pierwszym uruchomieniu wsq.5 zostanie ono usunięte z `user.json`.
- Jeśli to było logowanie do Steama, wsq.5 czyści też jego sesję w przeglądarce wbudowanej w Mantel. W tym wypadku konfiguracja pokaże się jeszcze raz.
- Bez kont nie ma osiągnięć ani historii gier Xbox oraz listy kupionych, ale niezainstalowanych gier.

**Pliki**
- `Mantel_0.5.4-wsq.5_x64-setup.exe`: instalator dla bieżącego użytkownika, bez uprawnień administratora.
- `Mantel_0.5.4-wsq.5_x64-portable.exe`: sam program, bez instalacji. Wymaga WebView2, który jest w Windows 11.

Pliki nie są podpisane cyfrowo, więc SmartScreen może pokazać ostrzeżenie. Wybierz wtedy *Więcej informacji → Uruchom mimo to*.

**VirusTotal (2026-11-17)**
- setup: 2/71 (APEX, Trapmine). To heurystyki ML, typowe dla nowego, niepodpisanego instalatora NSIS. [Raport](https://www.virustotal.com/gui/file/895f270fa5545e7bde9d13e173bd42b08e6730d7bd3bf81a24394a444ede8855)
- portable: 1/70 (Trapmine, heurystyka ML). [Raport](https://www.virustotal.com/gui/file/5bfdbfb7471d70b7271d5f2fb9cf5b59491d477f47dcf5408f0235a75637044a)

**SHA-256**
```
895f270fa5545e7bde9d13e173bd42b08e6730d7bd3bf81a24394a444ede8855  Mantel_0.5.4-wsq.5_x64-setup.exe
5bfdbfb7471d70b7271d5f2fb9cf5b59491d477f47dcf5408f0235a75637044a  Mantel_0.5.4-wsq.5_x64-portable.exe
```
