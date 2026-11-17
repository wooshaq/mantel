Mantel 0.5.4 WsQ, wersja 8. Zawiera wszystko z wsq.7.

**Nowe**
- **Przełączanie ekranu w centrum sterowania.** Pierwszy kafelek pokazuje, gdzie jest obraz: ekran wbudowany, ekran zewnętrzny, oba ekrany albo to samo na obu. Kliknięcie przełącza obraz między ekranem wbudowanym a zewnętrznym, tak jak Windows+P albo DisplaySwitch.exe, tylko bez uruchamiania osobnego programu.

**Lepsze**
- **Bez trailerów w tle, gdy Mantel jest w trayu albo zminimalizowany.** Trailer w tle się zatrzymuje, a animacje pauzują. Po powrocie z traya trailer rusza dopiero, gdy coś naciśniesz. Wygaszacz ekranu nie wlicza czasu spędzonego w trayu.

**Usunięte**
- **Stan Wi-Fi w centrum sterowania.** Przy połączeniu kablowym pokazywał „Brak połączenia”. Mantel nie korzysta już też z API Wi-Fi Windows.

**Pliki**
- `Mantel_0.5.4-wsq.8_x64-setup.exe`: instalator dla bieżącego użytkownika, bez uprawnień administratora.
- `Mantel_0.5.4-wsq.8_x64-portable.exe`: sam program, bez instalacji. Wymaga WebView2, który jest w Windows 11.

Pliki nie są podpisane cyfrowo, więc SmartScreen może pokazać ostrzeżenie. Wybierz wtedy *Więcej informacji → Uruchom mimo to*.

**VirusTotal (2026-11-17)**
- setup: 2/71 (APEX, Trapmine). To heurystyki ML, typowe dla niepodpisanego instalatora NSIS. [Raport](https://www.virustotal.com/gui/file/056aa92c9dd572c0899d3f61cfda6dccd04298d92543228a06ce6396fe79700c)
- portable: 1/71 (Trapmine, heurystyka ML). [Raport](https://www.virustotal.com/gui/file/b3ae1904c724e04fb2d60efca61c3a47a97484b6b4389a2a093eee2bfffca5d0)

**SHA-256**
```
056aa92c9dd572c0899d3f61cfda6dccd04298d92543228a06ce6396fe79700c  Mantel_0.5.4-wsq.8_x64-setup.exe
b3ae1904c724e04fb2d60efca61c3a47a97484b6b4389a2a093eee2bfffca5d0  Mantel_0.5.4-wsq.8_x64-portable.exe
```
