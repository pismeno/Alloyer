# Alloyer
Engine pro programovacké jazyky, funguje na základě 'nodes', které se zapisují do jsonu a jsou následně kompilovány do Rust kódu. Každá node referencuje buď jinou node která je zapsána v nodes.json jako funkce
anebo funkci z .dll pluginu, kterou plugin zaregistruje při kompilaci.
# Cíl
GUI desktopová aplikace, která vygeneruje json, a následně zavolá Alloyer engine. V enginu by mělo jít udělat jakýkoliv projekt.
# Do budoucna
- Možnost kompilovat buď do knihovny, nebo do executable.
# Knihovny, jazyky
Engine:
Rust, knihovny:
- serde_json, libloading
GUI:
Nejspíš C#, knihovny:
- Asi WPF
# Jak spustit?
Naklonujte si projekt, a použijte příkaz "cargo run", to zkompiluje .json soubory z 'nodes/' do Rust kódu v 'buildrs/src'. V repozitáři jsou nyní ukázkové soubory 'main.json' - standarní main funkce odkud kód běží, 'func.json' - ukázkové vlastní funkce vytvořené z již existujících nodes a 'nodes.json' - soubor kde se registrují právě vlastní funkce, je zde třeba uvést souboor, kde je funkce definována, její název, argumenty a jejich typy a return type. Aby ukázkové .json soubory fungovali, vložil jsem do classroomu i ukázkový plugin 'nodes.dll' (a jeho source kód), ten je třeba vložit v rootu projeku do složky 'plugins'.
