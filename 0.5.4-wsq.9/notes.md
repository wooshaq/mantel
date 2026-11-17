Mantel 0.5.4 WsQ, wersja 9: **statystyki**, historia gry i HowLongToBeat. Zawiera wszystko z wsq.8.

**Nowe**
- **Zakładka Statystyki**, pierwsza na górnej liście:
  - Kalendarz roku. Kolor kratki pokazuje, ile tego dnia grałeś, a liczba w kratce to osiągnięcia Steam odblokowane tego dnia.
  - Pod kalendarzem wybrany miesiąc dzień po dniu, słupki podzielone kolorami na gry.
  - Krzyżak przesuwa dzień, LT/RT zmienia miesiąc.
  - Na górze podsumowanie: czas w roku, najdłuższa seria, rekordowy dzień, osiągnięcia.
- **Historia gry** na stronie gry: każdy dzień grania z godziną i długością, przerwy typu „9 dni bez grania” i znaczek „Najdłuższy dzień”.
- **HowLongToBeat:**
  - Przycisk „Jak długo do końca” na stronie gry otwiera wyszukiwanie HLTB w przeglądarce. Wklejasz link do gry raz, potem Mantel sam odświeża czasy co dwa tygodnie.
  - Strona gry pokazuje fabułę, fabułę z dodatkami i 100%, a na nich zaznacza Twój czas.
  - Na ekranie głównym w miejscu „Na dysku” pojawia się „Do końca fabuły”.

**Lepsze**
- **Ekran główny:** w rzędzie mieści się 35 gier zamiast 18. Mantel rysuje tylko kafelki blisko ekranu, więc przewijanie jest lżejsze niż wcześniej.

**Warto wiedzieć**
- Kalendarz czasu gry i historia gry zaczynają się od tej wersji, bo wcześniej Mantel nie zapisywał pojedynczych sesji. Osiągnięcia w kalendarzu pokazują pełną historię z profilu Steam.
- HowLongToBeat nie ma API, a jego wyszukiwarka blokuje aplikacje, dlatego każdą grę łączysz linkiem. Mantel łączy się wyłącznie ze stronami gier, które sam połączysz.

**Pliki**
- `Mantel_0.5.4-wsq.9_x64-setup.exe`: instalator dla bieżącego użytkownika, bez uprawnień administratora.
- `Mantel_0.5.4-wsq.9_x64-portable.exe`: sam program, bez instalacji. Wymaga WebView2, który jest w Windows 11.

Pliki nie są podpisane cyfrowo, więc SmartScreen może pokazać ostrzeżenie. Wybierz wtedy *Więcej informacji → Uruchom mimo to*.

**VirusTotal (2026-11-17)**
- setup: 2/71 (APEX, Trapmine). To heurystyki ML, typowe dla niepodpisanego instalatora NSIS. [Raport](https://www.virustotal.com/gui/file/370a7fc03b9445c09a9e799b5aff17221352c2741676f36915019c3cf1027737)
- portable: 1/70 (Trapmine, heurystyka ML). [Raport](https://www.virustotal.com/gui/file/f0bc65151ebe61c8f729d4e0e535ce1fd7889fd50d48d4a173f424bf9dbce024)

**SHA-256**
```
370a7fc03b9445c09a9e799b5aff17221352c2741676f36915019c3cf1027737  Mantel_0.5.4-wsq.9_x64-setup.exe
f0bc65151ebe61c8f729d4e0e535ce1fd7889fd50d48d4a173f424bf9dbce024  Mantel_0.5.4-wsq.9_x64-portable.exe
```
