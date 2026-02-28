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
