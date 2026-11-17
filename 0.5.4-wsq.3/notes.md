Mantel 0.5.4 WsQ, wersja 3: **język polski** i poprawka konfiguracji. Zawiera wszystko z wsq.2 (brak PowerShella, aktualizatora i Trybu dewelopera).

**Nowe**
- Polski interfejs, z poprawną odmianą liczebników („1 gra”, „3 gry”, „5 gier”). Na polskim Windowsie włącza się sam, a sklep Steam w Mantelu też pokazuje się po polsku.

**Poprawione**
- Konfiguracja: gdy Windows był w języku, którego Mantel nie miał (np. polskim), wybranie „English” zostawiało następny krok pusty. Teraz krok wyświetla się od razu.
- Usunięta nieużywana wtyczka okien dialogowych (`tauri-plugin-dialog`) razem z jej uprawnieniem. Program ma przez to mniej zbędnego kodu.

**Pliki**
- `Mantel_0.5.4-wsq.3_x64-setup.exe`: instalator dla bieżącego użytkownika, bez uprawnień admina.
- `Mantel_0.5.4-wsq.3_x64-portable.exe`: sam program, bez instalacji (wymaga WebView2, który jest w Windows 11).

Pliki nie są podpisane cyfrowo, więc SmartScreen może pokazać ostrzeżenie: wybierz *Więcej informacji → Uruchom mimo to*.

**VirusTotal (2026-11-17)**
- setup: 2/71 (APEX, Trapmine). To heurystyki ML, typowe dla nowego, niepodpisanego instalatora NSIS – [raport](https://www.virustotal.com/gui/file/ee5f12d2cfbb77d7f393e734bfd2c07ba3bc2cafaa369c96c689445e7e73dd9a)
- portable: 1/59 (Trapmine, heurystyka ML) – [raport](https://www.virustotal.com/gui/file/3e49a10a73f2bd2d35a5ba3e59e4ff678b68bd00b625c1bffdc24379c3ba0c76)

**SHA-256**
```
ee5f12d2cfbb77d7f393e734bfd2c07ba3bc2cafaa369c96c689445e7e73dd9a  Mantel_0.5.4-wsq.3_x64-setup.exe
3e49a10a73f2bd2d35a5ba3e59e4ff678b68bd00b625c1bffdc24379c3ba0c76  Mantel_0.5.4-wsq.3_x64-portable.exe
```
