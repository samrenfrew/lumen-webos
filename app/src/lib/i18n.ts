// Short UI labels in nine languages. Missing entries fall back to the English text, so anything not listed (folder names, app names,
// the long option descriptions) is shown as written.

export const LANGS = [
	{code: 'en', name: 'English', locale: 'en-US'},
	{code: 'de', name: 'Deutsch', locale: 'de-DE'},
	{code: 'es', name: 'Español', locale: 'es-ES'},
	{code: 'fr', name: 'Français', locale: 'fr-FR'},
	{code: 'it', name: 'Italiano', locale: 'it-IT'},
	{code: 'pt', name: 'Português', locale: 'pt-BR'},
	{code: 'nl', name: 'Nederlands', locale: 'nl-NL'},
	{code: 'tr', name: 'Türkçe', locale: 'tr-TR'},
	{code: 'hi', name: 'हिन्दी', locale: 'hi-IN'}
];
const ORDER = ["de", "es", "fr", "it", "pt", "nl", "tr", "hi"];
const TABLE: Record<string, string[]> = {
	"Home Screen": ["Startbildschirm", "Pantalla de inicio", "Écran d'accueil", "Schermata Home", "Ecrã principal", "Startscherm", "Ana ekran", "होम स्क्रीन"],
	"Weather": ["Wetter", "Tiempo", "Météo", "Meteo", "Tempo", "Weer", "Hava durumu", "मौसम"],
	"Language and Clock": ["Sprache und Uhr", "Idioma y reloj", "Langue et horloge", "Lingua e orologio", "Idioma e relógio", "Taal en klok", "Dil ve saat", "भाषा और घड़ी"],
	"Language and Region": ["Sprache und Region", "Idioma y región", "Langue et région", "Lingua e regione", "Idioma e região", "Taal en regio", "Dil ve bölge", "भाषा और क्षेत्र"],
	"Photos": ["Fotos", "Fotos", "Photos", "Foto", "Fotos", "Foto's", "Fotoğraflar", "फ़ोटो"],
	"System": ["System", "Sistema", "Système", "Sistema", "Sistema", "Systeem", "Sistem", "सिस्टम"],
	"Choose wallpaper": ["Hintergrund wählen", "Elegir fondo", "Choisir le fond d'écran", "Scegli sfondo", "Escolher fundo", "Achtergrond kiezen", "Duvar kağıdı seç", "वॉलपेपर चुनें"],
	"Channels": ["Sender", "Canales", "Chaînes", "Canali", "Canais", "Zenders", "Kanallar", "चैनल"],
	"Appearance": ["Darstellung", "Apariencia", "Apparence", "Aspetto", "Aspeto", "Weergave", "Görünüm", "दिखावट"],
	"Theme": ["Thema", "Tema", "Thème", "Tema", "Tema", "Thema", "Tema", "थीम"],
	"Light": ["Hell", "Claro", "Clair", "Chiaro", "Claro", "Licht", "Açık", "लाइट"],
	"Dark": ["Dunkel", "Oscuro", "Sombre", "Scuro", "Escuro", "Donker", "Koyu", "डार्क"],
	"Weather country": ["Wetter-Land", "País del tiempo", "Pays météo", "Paese meteo", "País do tempo", "Weerland", "Hava durumu ülkesi", "मौसम देश"],
	"Hide the dock": ["Dock ausblenden", "Ocultar el dock", "Masquer le dock", "Nascondi il dock", "Ocultar o dock", "Dock verbergen", "Dock'u gizle", "डॉक छिपाएं"],
	"Video clip": ["Videoclip", "Clip de vídeo", "Clip vidéo", "Clip video", "Clip de vídeo", "Videoclip", "Video klibi", "वीडियो क्लिप"],
	"Shuffle": ["Zufall", "Aleatorio", "Aléatoire", "Casuale", "Aleatório", "Willekeurig", "Karışık", "शफल"],
	"On": ["Ein", "Activado", "Activé", "Attivo", "Ligado", "Aan", "Açık", "चालू"],
	"Dark mode": ["Dunkelmodus", "Modo oscuro", "Mode sombre", "Modalità scura", "Modo escuro", "Donkere modus", "Karanlık mod", "डार्क मोड"],
	"Apps": [
		"Apps",
		"Apps",
		"Applications",
		"App",
		"Apps",
		"Apps",
		"Uygulamalar",
		"ऐप्स"
	],
	"More apps": [
		"Weitere Apps",
		"Más apps",
		"Plus d'apps",
		"Altre app",
		"Mais apps",
		"Meer apps",
		"Daha fazla uygulama",
		"और ऐप्स"
	],
	"Recent": [
		"Zuletzt",
		"Recientes",
		"Récents",
		"Recenti",
		"Recentes",
		"Recent",
		"Son kullanılanlar",
		"हाल के"
	],
	"Sources": [
		"Quellen",
		"Fuentes",
		"Sources",
		"Sorgenti",
		"Fontes",
		"Bronnen",
		"Kaynaklar",
		"स्रोत"
	],
	"Inputs": [
		"Eingänge",
		"Entradas",
		"Entrées",
		"Ingressi",
		"Entradas",
		"Ingangen",
		"Girişler",
		"इनपुट"
	],
	"Control Center": [
		"Kontrollzentrum",
		"Centro de control",
		"Centre de contrôle",
		"Centro di controllo",
		"Central de Controle",
		"Bedieningscentrum",
		"Denetim Merkezi",
		"कंट्रोल सेंटर"
	],
	"Quick Controls": [
		"Schnellsteuerung",
		"Controles rápidos",
		"Commandes rapides",
		"Controlli rapidi",
		"Controles rápidos",
		"Snelbediening",
		"Hızlı Kontroller",
		"त्वरित नियंत्रण"
	],
	"Settings": [
		"Einstellungen",
		"Ajustes",
		"Réglages",
		"Impostazioni",
		"Configurações",
		"Instellingen",
		"Ayarlar",
		"सेटिंग्स"
	],
	"Start Screensaver": [
		"Bildschirmschoner starten",
		"Iniciar salvapantallas",
		"Lancer l'économiseur",
		"Avvia screensaver",
		"Iniciar protetor de ecrã",
		"Schermbeveiliging starten",
		"Ekran koruyucuyu başlat",
		"स्क्रीनसेवर शुरू करें"
	],
	"No apps selected — add some in Settings": [
		"Keine Apps ausgewählt – in den Einstellungen hinzufügen",
		"Sin apps: añade alguna en Ajustes",
		"Aucune app : ajoutez-en dans Réglages",
		"Nessuna app: aggiungine nelle Impostazioni",
		"Nenhuma app: adicione nas Definições",
		"Geen apps: voeg ze toe in Instellingen",
		"Uygulama yok — Ayarlar'dan ekleyin",
		"कोई ऐप नहीं — सेटिंग्स में जोड़ें"
	],
	"Wallpaper": [
		"Hintergrund",
		"Fondo de pantalla",
		"Fond d'écran",
		"Sfondo",
		"Papel de parede",
		"Achtergrond",
		"Duvar kâğıdı",
		"वॉलपेपर"
	],
	"Options": [
		"Optionen",
		"Opciones",
		"Options",
		"Opzioni",
		"Opções",
		"Opties",
		"Seçenekler",
		"विकल्प"
	],
	"Done": [
		"Fertig",
		"Listo",
		"Terminé",
		"Fatto",
		"Concluído",
		"Klaar",
		"Bitti",
		"हो गया"
	],
	"Re-order apps": [
		"Apps anordnen",
		"Reordenar apps",
		"Réorganiser les apps",
		"Riordina app",
		"Reordenar apps",
		"Apps herschikken",
		"Uygulamaları sırala",
		"ऐप्स क्रमबद्ध करें"
	],
	"Re-order sources": [
		"Quellen anordnen",
		"Reordenar fuentes",
		"Réorganiser les sources",
		"Riordina sorgenti",
		"Reordenar fontes",
		"Bronnen herschikken",
		"Kaynakları sırala",
		"स्रोत क्रमबद्ध करें"
	],
	"Nothing on the home screen yet.": [
		"Noch nichts auf dem Startbildschirm.",
		"Aún no hay nada en la pantalla de inicio.",
		"Rien sur l'écran d'accueil pour l'instant.",
		"Ancora niente nella schermata Home.",
		"Ainda nada no ecrã inicial.",
		"Nog niets op het startscherm.",
		"Ana ekranda henüz bir şey yok.",
		"होम स्क्रीन पर अभी कुछ नहीं है।"
	],
	"Move with the arrows · OK to drop · Hold OK for more": [
		"Mit Pfeilen verschieben · OK zum Ablegen · OK halten für mehr",
		"Mueve con las flechas · OK para soltar · Mantén OK para más",
		"Déplacez avec les flèches · OK pour déposer · Maintenez OK pour plus",
		"Sposta con le frecce · OK per rilasciare · Tieni premuto OK per altro",
		"Mova com as setas · OK para soltar · Mantenha OK para mais",
		"Verplaats met de pijlen · OK om neer te zetten · Houd OK vast voor meer",
		"Oklarla taşı · Bırakmak için OK · Daha fazlası için OK'a basılı tut",
		"तीरों से खिसकाएं · छोड़ने के लिए OK · और विकल्पों के लिए OK दबाए रखें"
	],
	"Uninstall App": ["App deinstallieren", "Desinstalar app", "Désinstaller l'app", "Disinstalla app", "Desinstalar app", "App verwijderen", "Uygulamayı kaldır", "ऐप अनइंस्टॉल करें"],
	"Press OK again to uninstall": ["Zum Deinstallieren erneut OK", "Pulsa OK otra vez para desinstalar", "Appuyez à nouveau sur OK pour désinstaller", "Premi di nuovo OK per disinstallare", "Prima OK de novo para desinstalar", "Druk nogmaals op OK om te verwijderen", "Kaldırmak için tekrar OK'a bas", "अनइंस्टॉल करने के लिए फिर OK दबाएं"],
	"Remove from Home Screen": ["Vom Startbildschirm entfernen", "Quitar de la pantalla de inicio", "Retirer de l'écran d'accueil", "Rimuovi dalla schermata Home", "Remover do ecrã inicial", "Van startscherm verwijderen", "Ana ekrandan kaldır", "होम स्क्रीन से हटाएं"],
	"Add to Folder": ["Zu Ordner hinzufügen", "Añadir a carpeta", "Ajouter à un dossier", "Aggiungi a cartella", "Adicionar a pasta", "Aan map toevoegen", "Klasöre ekle", "फ़ोल्डर में जोड़ें"],
	"Add to Games": [
		"Zu Spiele hinzufügen",
		"Añadir a Juegos",
		"Ajouter à Jeux",
		"Aggiungi a Giochi",
		"Adicionar a Jogos",
		"Toevoegen aan Games",
		"Oyunlar'a ekle",
		"गेम्स में जोड़ें"
	],
	"Add to Tools": [
		"Zu Tools hinzufügen",
		"Añadir a Herramientas",
		"Ajouter à Outils",
		"Aggiungi a Strumenti",
		"Adicionar a Ferramentas",
		"Toevoegen aan Tools",
		"Araçlar'a ekle",
		"टूल्स में जोड़ें"
	],
	"Add to Media": [
		"Zu Medien hinzufügen",
		"Añadir a Multimedia",
		"Ajouter à Médias",
		"Aggiungi a Media",
		"Adicionar a Multimédia",
		"Toevoegen aan Media",
		"Medya'ya ekle",
		"मीडिया में जोड़ें"
	],
	"Take out of Folder": [
		"Aus Ordner nehmen",
		"Sacar de la carpeta",
		"Sortir du dossier",
		"Togli dalla cartella",
		"Tirar da pasta",
		"Uit map halen",
		"Klasörden çıkar",
		"फ़ोल्डर से निकालें"
	],
	"Sound mode": [
		"Klangmodus",
		"Modo de sonido",
		"Mode son",
		"Modalità audio",
		"Modo de som",
		"Geluidsmodus",
		"Ses modu",
		"साउंड मोड"
	],
	"Picture": [
		"Bild",
		"Imagen",
		"Image",
		"Immagine",
		"Imagem",
		"Beeld",
		"Resim",
		"चित्र"
	],
	"Sleep timer": [
		"Sleep-Timer",
		"Temporizador",
		"Minuterie",
		"Timer di spegnimento",
		"Temporizador",
		"Slaaptimer",
		"Uyku zamanlayıcı",
		"स्लीप टाइमर"
	],
	"Screensaver": [
		"Bildschirmschoner",
		"Salvapantallas",
		"Économiseur d'écran",
		"Screensaver",
		"Protetor de ecrã",
		"Schermbeveiliging",
		"Ekran koruyucu",
		"स्क्रीनसेवर"
	],
	"Art mode": [
		"Kunstmodus",
		"Modo arte",
		"Mode art",
		"Modalità arte",
		"Modo arte",
		"Kunstmodus",
		"Sanat modu",
		"आर्ट मोड"
	],
	"Start now": [
		"Jetzt starten",
		"Iniciar ya",
		"Lancer",
		"Avvia ora",
		"Iniciar agora",
		"Nu starten",
		"Şimdi başlat",
		"अभी शुरू करें"
	],
	"Screen off": [
		"Bildschirm aus",
		"Pantalla off",
		"Écran éteint",
		"Schermo spento",
		"Ecrã desligado",
		"Scherm uit",
		"Ekranı kapat",
		"स्क्रीन बंद"
	],
	"Restart": [
		"Neustart",
		"Reiniciar",
		"Redémarrer",
		"Riavvia",
		"Reiniciar",
		"Herstarten",
		"Yeniden başlat",
		"रीस्टार्ट"
	],
	"Press OK again": [
		"OK erneut drücken",
		"Pulsa OK otra vez",
		"Appuyez encore sur OK",
		"Premi di nuovo OK",
		"Prima OK novamente",
		"Druk nogmaals OK",
		"Tekrar OK'a basın",
		"फिर OK दबाएं"
	],
	"Backlight": ["Hintergrundbeleuchtung", "Luz de fondo", "Rétroéclairage", "Retroilluminazione", "Brilho OLED", "Achtergrondverlichting", "Arka ışık", "बैकलाइट"],
	"Eye Comfort": ["Augenkomfort", "Confort visual", "Confort visuel", "Comfort visivo", "Conforto visual", "Oogcomfort", "Göz konforu", "आई कम्फर्ट"],
	"Volume": [
		"Lautstärke",
		"Volumen",
		"Volume",
		"Volume",
		"Volume",
		"Volume",
		"Ses",
		"वॉल्यूम"
	],
	"Muted": [
		"Stumm",
		"Silenciado",
		"Muet",
		"Muto",
		"Sem som",
		"Gedempt",
		"Sessiz",
		"म्यूट"
	],
	"Off": [
		"Aus",
		"Apagado",
		"Désactivé",
		"Spento",
		"Desligado",
		"Uit",
		"Kapalı",
		"बंद"
	],
	"Offline": [
		"Offline",
		"Sin conexión",
		"Hors ligne",
		"Offline",
		"Offline",
		"Offline",
		"Çevrimdışı",
		"ऑफ़लाइन"
	],
	"After {n} min": [
		"Nach {n} Min.",
		"Tras {n} min",
		"Après {n} min",
		"Dopo {n} min",
		"Após {n} min",
		"Na {n} min",
		"{n} dk sonra",
		"{n} मिनट बाद"
	],
	"{n} min": [
		"{n} Min.",
		"{n} min",
		"{n} min",
		"{n} min",
		"{n} min",
		"{n} min",
		"{n} dk",
		"{n} मिनट"
	],
	"Live app list via Homebrew Channel": [
		"Live-App-Liste über Homebrew Channel",
		"Lista de apps en vivo (Homebrew Channel)",
		"Liste d'apps en direct (Homebrew Channel)",
		"Elenco app live (Homebrew Channel)",
		"Lista de apps em direto (Homebrew Channel)",
		"Live app-lijst via Homebrew Channel",
		"Homebrew Channel ile canlı uygulama listesi",
		"Homebrew Channel से लाइव ऐप सूची"
	],
	"Video wallpaper": [
		"Video-Hintergrund",
		"Fondo de vídeo",
		"Fond vidéo",
		"Sfondo video",
		"Papel de parede em vídeo",
		"Videoachtergrond",
		"Video duvar kâğıdı",
		"वीडियो वॉलपेपर"
	],
	"Show system apps": [
		"Systemapps anzeigen",
		"Mostrar apps del sistema",
		"Afficher les apps système",
		"Mostra app di sistema",
		"Mostrar apps do sistema",
		"Systeem-apps tonen",
		"Sistem uygulamalarını göster",
		"सिस्टम ऐप्स दिखाएं"
	],
	"Date and weather": [
		"Datum und Wetter",
		"Fecha y tiempo",
		"Date et météo",
		"Data e meteo",
		"Data e meteorologia",
		"Datum en weer",
		"Tarih ve hava durumu",
		"तारीख और मौसम"
	],
	"Weather city": [
		"Wetter-Ort",
		"Ciudad del tiempo",
		"Ville météo",
		"Città meteo",
		"Cidade do tempo",
		"Weerstad",
		"Hava durumu şehri",
		"मौसम शहर"
	],
	"Change wallpaper automatically": [
		"Hintergrund automatisch wechseln",
		"Cambiar fondo automáticamente",
		"Changer le fond automatiquement",
		"Cambia sfondo automaticamente",
		"Mudar papel de parede automaticamente",
		"Achtergrond automatisch wisselen",
		"Duvar kâğıdını otomatik değiştir",
		"वॉलपेपर अपने आप बदलें"
	],
	"Dim the wallpaper": [
		"Hintergrund abdunkeln",
		"Atenuar fondo",
		"Assombrir le fond",
		"Scurisci lo sfondo",
		"Escurecer papel de parede",
		"Achtergrond dimmen",
		"Duvar kâğıdını karart",
		"वॉलपेपर मंद करें"
	],
	"At night": ["Nachts", "De noche", "La nuit", "Di notte", "À noite", "'s Nachts", "Geceleri", "रात में"],
	"Wallpaper captions": ["Hintergrund-Beschriftungen", "Rótulos del fondo", "Légendes du fond d'écran", "Didascalie dello sfondo", "Legendas do fundo", "Achtergrondbijschriften", "Duvar kağıdı açıklamaları", "वॉलपेपर कैप्शन"],
	"Dim when the dock is hidden": [
		"Abdunkeln, wenn das Dock ausgeblendet ist",
		"Atenuar con el dock oculto",
		"Assombrir quand le dock est masqué",
		"Scurisci con il dock nascosto",
		"Escurecer com o dock oculto",
		"Dimmen als het dock verborgen is",
		"Dock gizliyken karart",
		"डॉक छिपने पर मंद करें"
	],
	"AM/PM on the clock": [
		"AM/PM an der Uhr",
		"AM/PM en el reloj",
		"AM/PM sur l'horloge",
		"AM/PM sull'orologio",
		"AM/PM no relógio",
		"AM/PM op de klok",
		"Saatte AM/PM",
		"घड़ी में AM/PM"
	],
	"Clock": [
		"Uhr",
		"Reloj",
		"Horloge",
		"Orologio",
		"Relógio",
		"Klok",
		"Saat",
		"घड़ी"
	],
	"Language": [
		"Sprache",
		"Idioma",
		"Langue",
		"Lingua",
		"Idioma",
		"Taal",
		"Dil",
		"भाषा"
	],
	"Auto": [
		"Automatisch",
		"Automático",
		"Auto",
		"Automatico",
		"Automático",
		"Automatisch",
		"Otomatik",
		"स्वचालित"
	],
	"12-hour": [
		"12 Stunden",
		"12 horas",
		"12 heures",
		"12 ore",
		"12 horas",
		"12 uur",
		"12 saat",
		"12 घंटे"
	],
	"24-hour": [
		"24 Stunden",
		"24 horas",
		"24 heures",
		"24 ore",
		"24 horas",
		"24 uur",
		"24 saat",
		"24 घंटे"
	],
	"Clear": [
		"Klar",
		"Despejado",
		"Dégagé",
		"Sereno",
		"Limpo",
		"Helder",
		"Açık",
		"साफ"
	],
	"Partly cloudy": [
		"Teils bewölkt",
		"Parcialmente nublado",
		"Partiellement nuageux",
		"Parzialmente nuvoloso",
		"Parcialmente nublado",
		"Half bewolkt",
		"Parçalı bulutlu",
		"आंशिक बादल"
	],
	"Cloudy": [
		"Bewölkt",
		"Nublado",
		"Nuageux",
		"Nuvoloso",
		"Nublado",
		"Bewolkt",
		"Bulutlu",
		"बादल"
	],
	"Fog": [
		"Nebel",
		"Niebla",
		"Brouillard",
		"Nebbia",
		"Nevoeiro",
		"Mist",
		"Sis",
		"कोहरा"
	],
	"Rain": [
		"Regen",
		"Lluvia",
		"Pluie",
		"Pioggia",
		"Chuva",
		"Regen",
		"Yağmur",
		"बारिश"
	],
	"Snow": [
		"Schnee",
		"Nieve",
		"Neige",
		"Neve",
		"Neve",
		"Sneeuw",
		"Kar",
		"बर्फ"
	],
	"High": ["Max.", "Máx.", "Max.", "Max", "Máx.", "Max.", "En yüksek", "अधिकतम"],
	"Low": ["Min.", "Mín.", "Min.", "Min", "Mín.", "Min.", "En düşük", "न्यूनतम"],
	"Rain from": ["Regen ab", "Lluvia desde las", "Pluie dès", "Pioggia dalle", "Chuva a partir das", "Regen vanaf", "Yağmur başlangıcı", "बारिश"],
	"Showers": [
		"Schauer",
		"Chubascos",
		"Averses",
		"Rovesci",
		"Aguaceiros",
		"Buien",
		"Sağanak",
		"बौछारें"
	],
	"Storm": [
		"Gewitter",
		"Tormenta",
		"Orage",
		"Temporale",
		"Tempestade",
		"Onweer",
		"Fırtına",
		"तूफ़ान"
	],
	"OK adds or removes an item · Hold OK to re-order · Back returns": [
		"OK fügt hinzu oder entfernt · OK halten zum Anordnen · Zurück zum Start",
		"OK añade o quita · Mantén OK para reordenar · Atrás vuelve al inicio",
		"OK ajoute ou retire · Maintenez OK pour réorganiser · Retour revient à l'accueil",
		"OK aggiunge o rimuove · Tieni OK per riordinare · Indietro torna alla Home",
		"OK adiciona ou remove · Mantenha OK para reordenar · Voltar regressa ao início",
		"OK voegt toe of verwijdert · Houd OK ingedrukt om te herschikken · Terug naar start",
		"OK ekler veya kaldırır · Sıralamak için OK'a basılı tutun · Geri ana ekrana döner",
		"OK से जोड़ें या हटाएं · क्रम बदलने के लिए OK दबाए रखें · वापस जाने के लिए Back"
	],
	"OK picks up an item · Hold OK or choose Done to finish": [
		"OK nimmt auf · OK halten oder Fertig zum Beenden",
		"OK coge un elemento · Mantén OK o elige Listo para terminar",
		"OK saisit un élément · Maintenez OK ou choisissez Terminé",
		"OK prende un elemento · Tieni OK o scegli Fatto per finire",
		"OK agarra um item · Mantenha OK ou escolha Concluído",
		"OK pakt een item op · Houd OK ingedrukt of kies Klaar",
		"OK öğeyi alır · Bitirmek için OK'a basılı tutun veya Bitti'yi seçin",
		"OK से आइटम उठाएं · खत्म करने के लिए OK दबाए रखें या हो गया चुनें"
	],
	"Arrows move it · OK puts it down · Back cancels": [
		"Pfeile bewegen · OK legt ab · Zurück bricht ab",
		"Flechas mueven · OK suelta · Atrás cancela",
		"Flèches pour déplacer · OK pour poser · Retour annule",
		"Frecce per spostare · OK per posare · Indietro annulla",
		"Setas movem · OK pousa · Voltar cancela",
		"Pijlen verplaatsen · OK zet neer · Terug annuleert",
		"Oklar taşır · OK bırakır · Geri iptal eder",
		"तीर खिसकाते हैं · OK छोड़ता है · Back रद्द करता है"
	],
};

let current = 'en';
let clockMode = 'auto';

/** 'auto' follows the TV (webOS sets navigator.language to its menu language) */
export function setLang (pref: string): void {
	const base = (pref === 'auto' ? (navigator.language || 'en') : pref).slice(0, 2).toLowerCase();
	current = LANGS.some((l) => l.code === base) ? base : 'en';
}
export function langLocale (): string {
	return LANGS.filter((l) => l.code === current)[0].locale;
}
export function isDark (mode: string, now = new Date()): boolean {
	const h = now.getHours();
	return mode === 'on' || (mode === 'auto' && (h >= 19 || h < 6));
}
export function setClockMode (mode: string): void { clockMode = mode; }
export function getClockMode (): string { return clockMode; }

export function t (text: string, vars?: {n: number}): string {
	const row = TABLE[text];
	const at = ORDER.indexOf(current);
	let out = row && at >= 0 && row[at] ? row[at] : text;
	if (vars) out = out.replace('{n}', String(vars.n));
	return out;
}
