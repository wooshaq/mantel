Mantel 0.5.4 WsQ, wersja 4. Dochodzą: ikona w zasobniku, osiągnięcia Steam bez logowania, Big Picture i HDR na czas gry. Wersja zawiera wszystko z wsq.3: język polski, brak PowerShella, aktualizatora i Trybu dewelopera.

**Nowe**
- **Ikona w zasobniku** (Ustawienia → System). Minimalizacja albo zamknięcie okna chowa Mantel obok zegara. Kliknięcie ikony przywraca okno, a z jej menu można zamknąć program.
- **Osiągnięcia Steam bez logowania** (Ustawienia → Konta → Osiągnięcia Steam).
  - Mantel czyta je z publicznego profilu konta, na które Steam jest zalogowany na tym komputerze. Można też wkleić link do innego profilu.
  - Liczniki pokazują się przy grach, w które grałeś. Strona gry ma pełną listę z datą odblokowania każdego osiągnięcia.
  - Szczegóły gier w profilu muszą być publiczne.
- **Gry Steam w Big Picture** (Ustawienia → System). Steam otwiera Big Picture przed grą i zamyka je po niej.
- **HDR na czas gry.**
  - Na stronie gry jest przycisk „HDR”. Mantel włącza HDR przed startem gry i wyłącza po jej zakończeniu, ale tylko na ekranach, które sam przełączył.
  - Sposób włączania wybierasz w Ustawienia → System: Windows albo sterownik NVIDIA.

**Poprawione**
- Gdy szyfrowanie Windows (DPAPI) zawiedzie, tokeny kont nie trafiają na dysk jawnym tekstem. W takim przypadku trzeba się po prostu zalogować ponownie.

**Warto wiedzieć**
- HDR w trybie NVIDIA wymaga sterownika NVIDIA. Na ROG Ally (AMD) wybierz Windows.
- Osiągnięcia z profilu odświeżają się co kilka godzin, a po zakończeniu gry Steam po około 30 sekundach.

**Pliki**
- `Mantel_0.5.4-wsq.4_x64-setup.exe`: instalator dla bieżącego użytkownika, bez uprawnień administratora.
- `Mantel_0.5.4-wsq.4_x64-portable.exe`: sam program, bez instalacji. Wymaga WebView2, który jest w Windows 11.

Pliki nie są podpisane cyfrowo, więc SmartScreen może pokazać ostrzeżenie. Wybierz wtedy *Więcej informacji → Uruchom mimo to*.

**VirusTotal (2026-11-17)**
- setup: 2/71 (APEX, Trapmine). To heurystyki ML, typowe dla nowego, niepodpisanego instalatora NSIS. [Raport](https://www.virustotal.com/gui/file/b89f7a001b4b94a01b7c983a40f6100d38b5df764e2723977add80bc4d64e1f7)
- portable: 1/71 (Trapmine, heurystyka ML). [Raport](https://www.virustotal.com/gui/file/8b7c65b7e822e4ab42555c9c31b7d623b69d5e930fedbbf3f72f49aac903d0f2)

**SHA-256**
```
b89f7a001b4b94a01b7c983a40f6100d38b5df764e2723977add80bc4d64e1f7  Mantel_0.5.4-wsq.4_x64-setup.exe
8b7c65b7e822e4ab42555c9c31b7d623b69d5e930fedbbf3f72f49aac903d0f2  Mantel_0.5.4-wsq.4_x64-portable.exe
```
