Mantel 0.5.4 WsQ, wersja 19. Wyłączanie komputera bez uruchamiania shutdown.exe. Zawiera wszystko z wsq.18.

**Poprawione**
- **„Wyłącz” i „Uruchom ponownie” prosi teraz o to bezpośrednio Windows**, tak jak menu Start, i nie uruchamia przy tym programu `shutdown.exe`. Bitdefender uznał wcześniejszy sposób (niepodpisany program w tle uruchamiający `shutdown` po samej nazwie) za złośliwe działanie.
  - Nic nie jest wymuszane: inne programy zamykają się normalnie i mogą zapytać o zapisanie zmian.
- **Otwieranie folderu** uruchamia Eksplorator po pełnej ścieżce w folderze Windows, a nie po samej nazwie.

**Dobrze wiedzieć**
- Jeśli Bitdefender usunął wcześniej wpis autostartu Mantela (*Powiadomienia → „Potencjalnie niechciany obiekt został poddany kwarantannie”*, ścieżka `...\CurrentVersion\Run\mantel`), kliknij tam „Przywróć” albo wyłącz i włącz autostart w ustawieniach Mantela.
- Bitdefender może nadal oceniać niepodpisany program podejrzliwie. Pewnym rozwiązaniem jest dopiero zgłoszenie fałszywego alarmu albo podpis cyfrowy.

**Pliki**
- `Mantel_0.5.4-wsq.19_x64-setup.exe`: instalator dla bieżącego użytkownika, bez uprawnień administratora.
- `Mantel_0.5.4-wsq.19_x64-portable.exe`: sam program, bez instalacji. Wymaga WebView2, który jest w Windows 11.

Pliki nie są podpisane cyfrowo, więc SmartScreen może pokazać ostrzeżenie. Wybierz wtedy *Więcej informacji → Uruchom mimo to*.

**VirusTotal (2026-11-23)**
- setup: 2/69 (APEX, Trapmine). To heurystyki ML, typowe dla niepodpisanego instalatora NSIS. [Raport](https://www.virustotal.com/gui/file/2d2ab76a3bcf36aee7b578d24795bf6deef59efb3a52a03248f629f466fd7c49)
- portable: 1/71 (Trapmine, heurystyka ML). [Raport](https://www.virustotal.com/gui/file/6c89d6dccf77ef324d63d7b1f71b1520f874e344c66cab1d61ee7a78f6830e1f)

**SHA-256**
```
2d2ab76a3bcf36aee7b578d24795bf6deef59efb3a52a03248f629f466fd7c49  Mantel_0.5.4-wsq.19_x64-setup.exe
6c89d6dccf77ef324d63d7b1f71b1520f874e344c66cab1d61ee7a78f6830e1f  Mantel_0.5.4-wsq.19_x64-portable.exe
```
