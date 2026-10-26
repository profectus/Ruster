#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Locale {
    En,
    Cs,
    De,
    Es,
    Fr,
    It,
    Pl,
    Pt,
    Zh,
    Ja,
}

impl Locale {
    pub const ALL: &'static [Locale] = &[
        Locale::En,
        Locale::Cs,
        Locale::De,
        Locale::Es,
        Locale::Fr,
        Locale::It,
        Locale::Pl,
        Locale::Pt,
        Locale::Zh,
        Locale::Ja,
    ];

    pub fn as_code(&self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::Cs => "cs",
            Locale::De => "de",
            Locale::Es => "es",
            Locale::Fr => "fr",
            Locale::It => "it",
            Locale::Pl => "pl",
            Locale::Pt => "pt",
            Locale::Zh => "zh",
            Locale::Ja => "ja",
        }
    }

    pub fn from_code(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "cs" | "cz" | "czech" => Locale::Cs,
            "de" | "german" => Locale::De,
            "es" | "spanish" => Locale::Es,
            "fr" | "french" => Locale::Fr,
            "it" | "italian" => Locale::It,
            "pl" | "polish" => Locale::Pl,
            "pt" | "portuguese" => Locale::Pt,
            "zh" | "cn" | "chinese" => Locale::Zh,
            "ja" | "jp" | "japanese" => Locale::Ja,
            _ => Locale::En,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Locale::En => "English",
            Locale::Cs => "Čeština",
            Locale::De => "Deutsch",
            Locale::Es => "Español",
            Locale::Fr => "Français",
            Locale::It => "Italiano",
            Locale::Pl => "Polski",
            Locale::Pt => "Português",
            Locale::Zh => "简体中文",
            Locale::Ja => "日本語",
        }
    }

    pub fn english_name(&self) -> &'static str {
        match self {
            Locale::En => "English",
            Locale::Cs => "Czech",
            Locale::De => "German",
            Locale::Es => "Spanish",
            Locale::Fr => "French",
            Locale::It => "Italian",
            Locale::Pl => "Polish",
            Locale::Pt => "Portuguese",
            Locale::Zh => "Chinese (Simplified)",
            Locale::Ja => "Japanese",
        }
    }

    pub fn flag(&self) -> &'static str {
        match self {
            Locale::En => "🇺🇸",
            Locale::Cs => "🇨🇿",
            Locale::De => "🇩🇪",
            Locale::Es => "🇪🇸",
            Locale::Fr => "🇫🇷",
            Locale::It => "🇮🇹",
            Locale::Pl => "🇵🇱",
            Locale::Pt => "🇵🇹",
            Locale::Zh => "🇨🇳",
            Locale::Ja => "🇯🇵",
        }
    }

    #[allow(dead_code)]
    pub fn next(&self) -> Self {
        match self {
            Locale::En => Locale::Cs,
            Locale::Cs => Locale::De,
            Locale::De => Locale::Es,
            Locale::Es => Locale::Fr,
            Locale::Fr => Locale::It,
            Locale::It => Locale::Pl,
            Locale::Pl => Locale::Pt,
            Locale::Pt => Locale::Zh,
            Locale::Zh => Locale::Ja,
            Locale::Ja => Locale::En,
        }
    }
}

#[derive(Clone, Copy)]
pub struct I18n {
    pub locale: Locale,
}

#[allow(dead_code)]
impl I18n {
    pub const fn new(locale: Locale) -> Self {
        Self { locale }
    }

    // --- App & Menu Bar ---
    pub fn menu_connections(&self) -> &'static str {
        match self.locale {
            Locale::En => "Connections",
            Locale::Cs => "Spojení",
            Locale::De => "Verbindungen",
            Locale::Es => "Conexiones",
            Locale::Fr => "Connexions",
            Locale::It => "Connessioni",
            Locale::Pl => "Połączenia",
            Locale::Pt => "Conexões",
            Locale::Zh => "连接",
            Locale::Ja => "接続",
        }
    }

    pub fn menu_view(&self) -> &'static str {
        match self.locale {
            Locale::En => "View",
            Locale::Cs => "Zobrazení",
            Locale::De => "Ansicht",
            Locale::Es => "Ver",
            Locale::Fr => "Présentation",
            Locale::It => "Vista",
            Locale::Pl => "Widok",
            Locale::Pt => "Visualizar",
            Locale::Zh => "视图",
            Locale::Ja => "表示",
        }
    }

    pub fn menu_actions(&self) -> &'static str {
        match self.locale {
            Locale::En => "Actions",
            Locale::Cs => "Akce",
            Locale::De => "Aktionen",
            Locale::Es => "Acciones",
            Locale::Fr => "Actions",
            Locale::It => "Azioni",
            Locale::Pl => "Akcje",
            Locale::Pt => "Ações",
            Locale::Zh => "操作",
            Locale::Ja => "アクション",
        }
    }

    pub fn menu_site_manager(&self) -> &'static str {
        match self.locale {
            Locale::En => "Connection Manager...",
            Locale::Cs => "Správce spojení...",
            Locale::De => "Verbindungsmanager...",
            Locale::Es => "Gestor de conexiones...",
            Locale::Fr => "Gestionnaire de connexions...",
            Locale::It => "Gestione connessioni...",
            Locale::Pl => "Menedżer połączeń...",
            Locale::Pt => "Gerenciador de conexões...",
            Locale::Zh => "站点管理器...",
            Locale::Ja => "接続マネージャー...",
        }
    }

    pub fn menu_quick_filter(&self) -> &'static str {
        match self.locale {
            Locale::En => "Quick Filter...",
            Locale::Cs => "Rychlý filtr...",
            Locale::De => "Schnellfilter...",
            Locale::Es => "Filtro rápido...",
            Locale::Fr => "Filtre rapide...",
            Locale::It => "Filtro rapido...",
            Locale::Pl => "Szybki filtr...",
            Locale::Pt => "Filtro rápido...",
            Locale::Zh => "快速过滤...",
            Locale::Ja => "クイックフィルター...",
        }
    }

    pub fn menu_toggle_dual_pane(&self) -> &'static str {
        match self.locale {
            Locale::En => "Toggle Local Pane",
            Locale::Cs => "Přepnout lokální panel",
            Locale::De => "Lokales Fenster umschalten",
            Locale::Es => "Alternar panel local",
            Locale::Fr => "Basculer panneau local",
            Locale::It => "Attiva/disattiva pannello locale",
            Locale::Pl => "Przełącz panel lokalny",
            Locale::Pt => "Alternar painel local",
            Locale::Zh => "切换本地面板",
            Locale::Ja => "ローカルパネル切り替え",
        }
    }

    pub fn menu_swap_panes(&self) -> &'static str {
        match self.locale {
            Locale::En => "Swap Left and Right Panes",
            Locale::Cs => "Prohodit levou a pravou stranu",
            Locale::De => "Seiten vertauschen",
            Locale::Es => "Intercambiar paneles",
            Locale::Fr => "Inverser les panneaux",
            Locale::It => "Scambia pannelli",
            Locale::Pl => "Zamień panele miejscami",
            Locale::Pt => "Alternar lados dos painéis",
            Locale::Zh => "左右面板对调",
            Locale::Ja => "左右パネルを入れ替え",
        }
    }

    pub fn menu_toggle_local_tree(&self) -> &'static str {
        match self.locale {
            Locale::En => "Toggle Local Directory Tree",
            Locale::Cs => "Přepnout lokální strom složek",
            Locale::De => "Lokalen Ordnerbaum umschalten",
            Locale::Es => "Alternar árbol local",
            Locale::Fr => "Arborescence locale",
            Locale::It => "Albero directory locale",
            Locale::Pl => "Przełącz lokalne drzewo",
            Locale::Pt => "Alternar árvore local",
            Locale::Zh => "切换本地目录树",
            Locale::Ja => "ローカルツリー切り替え",
        }
    }

    pub fn menu_toggle_remote_tree(&self) -> &'static str {
        match self.locale {
            Locale::En => "Toggle Remote Directory Tree",
            Locale::Cs => "Přepnout vzdálený strom složek",
            Locale::De => "Entfernten Ordnerbaum umschalten",
            Locale::Es => "Alternar árbol remoto",
            Locale::Fr => "Arborescence distante",
            Locale::It => "Albero directory remota",
            Locale::Pl => "Przełącz zdalne drzewo",
            Locale::Pt => "Alternar árvore remota",
            Locale::Zh => "切换远程目录树",
            Locale::Ja => "リモートツリー切り替え",
        }
    }

    pub fn menu_toggle_hidden(&self) -> &'static str {
        match self.locale {
            Locale::En => "Show Hidden Files",
            Locale::Cs => "Zobrazit skryté soubory",
            Locale::De => "Versteckte Dateien anzeigen",
            Locale::Es => "Mostrar archivos ocultos",
            Locale::Fr => "Afficher fichiers cachés",
            Locale::It => "Mostra file nascosti",
            Locale::Pl => "Pokaż ukryte pliki",
            Locale::Pt => "Mostrar arquivos ocultos",
            Locale::Zh => "显示隐藏文件",
            Locale::Ja => "隠しファイルを表示",
        }
    }

    pub fn menu_toggle_queue(&self) -> &'static str {
        match self.locale {
            Locale::En => "Show Transfer Queue",
            Locale::Cs => "Zobrazit panel přenosů",
            Locale::De => "Warteschlange anzeigen",
            Locale::Es => "Mostrar cola de transferencias",
            Locale::Fr => "Afficher file d'attente",
            Locale::It => "Mostra coda trasferimenti",
            Locale::Pl => "Pokaż kolejkę transferu",
            Locale::Pt => "Mostrar fila de transferências",
            Locale::Zh => "显示传输队列",
            Locale::Ja => "転送キューを表示",
        }
    }

    pub fn menu_refresh(&self) -> &'static str {
        match self.locale {
            Locale::En => "Refresh Panes",
            Locale::Cs => "Obnovit panely",
            Locale::De => "Aktualisieren",
            Locale::Es => "Actualizar",
            Locale::Fr => "Actualiser",
            Locale::It => "Aggiorna",
            Locale::Pl => "Odśwież",
            Locale::Pt => "Atualizar",
            Locale::Zh => "刷新面板",
            Locale::Ja => "最新の情報に更新",
        }
    }

    pub fn menu_switch_language(&self) -> &'static str {
        match self.locale {
            Locale::En => "Language Settings...",
            Locale::Cs => "Nastavení jazyka...",
            Locale::De => "Spracheinstellungen...",
            Locale::Es => "Configuración de idioma...",
            Locale::Fr => "Paramètres de langue...",
            Locale::It => "Impostazioni lingua...",
            Locale::Pl => "Ustawienia języka...",
            Locale::Pt => "Configurações de idioma...",
            Locale::Zh => "语言设置...",
            Locale::Ja => "言語設定...",
        }
    }

    pub fn menu_toggle_theme(&self) -> &'static str {
        match self.locale {
            Locale::En => "Toggle Dark / Light Theme",
            Locale::Cs => "Přepnout tmavý / světlý motiv",
            Locale::De => "Dunkles / Helles Design umschalten",
            Locale::Es => "Cambiar tema oscuro / claro",
            Locale::Fr => "Basculer thème sombre / clair",
            Locale::It => "Cambia tema scuro / chiaro",
            Locale::Pl => "Przełącz ciemny / jasny motyw",
            Locale::Pt => "Alternar tema escuro / claro",
            Locale::Zh => "切换深色 / 浅色主题",
            Locale::Ja => "ダーク / ライトテーマ切り替え",
        }
    }

    // --- Appearance & Theme ---
    pub fn theme_section_title(&self) -> &'static str {
        match self.locale {
            Locale::En => "APPEARANCE & THEME",
            Locale::Cs => "VZHLED A MOTIV",
            Locale::De => "ERSCHEINUNGSBILD & DESIGN",
            Locale::Es => "ASPECTO Y TEMA",
            Locale::Fr => "APPARENCE & THÈME",
            Locale::It => "ASPETTO E TEMA",
            Locale::Pl => "WYGLĄD I MOTYW",
            Locale::Pt => "APARÊNCIA E TEMA",
            Locale::Zh => "外观与主题",
            Locale::Ja => "外観とテーマ",
        }
    }

    pub fn theme_light_label(&self) -> &'static str {
        match self.locale {
            Locale::En => "Light",
            Locale::Cs => "Světlý",
            Locale::De => "Hell",
            Locale::Es => "Claro",
            Locale::Fr => "Clair",
            Locale::It => "Chiaro",
            Locale::Pl => "Jasny",
            Locale::Pt => "Claro",
            Locale::Zh => "浅色",
            Locale::Ja => "ライト",
        }
    }

    pub fn theme_dark_label(&self) -> &'static str {
        match self.locale {
            Locale::En => "Dark",
            Locale::Cs => "Tmavý",
            Locale::De => "Dunkel",
            Locale::Es => "Oscuro",
            Locale::Fr => "Sombre",
            Locale::It => "Scuro",
            Locale::Pl => "Ciemny",
            Locale::Pt => "Escuro",
            Locale::Zh => "深色",
            Locale::Ja => "ダーク",
        }
    }

    pub fn theme_desc(&self) -> &'static str {
        match self.locale {
            Locale::En => "Choose between Light and Dark theme",
            Locale::Cs => "Vyberte si mezi světlým a tmavým motivem",
            Locale::De => "Wählen Sie zwischen hellem und dunklem Design",
            Locale::Es => "Elija entre tema claro u oscuro",
            Locale::Fr => "Choisissez entre le thème clair et sombre",
            Locale::It => "Scegli tra tema chiaro o scuro",
            Locale::Pl => "Wybierz między jasnym a ciemnym motywem",
            Locale::Pt => "Escolha entre o tema claro e escuro",
            Locale::Zh => "在浅色和深色主题之间选择",
            Locale::Ja => "ライトとダークのテーマを選択",
        }
    }

    pub fn theme_light_desc(&self) -> &'static str {
        match self.locale {
            Locale::En => "Default clear appearance",
            Locale::Cs => "Výchozí světlý vzhled",
            Locale::De => "Standardmäßiges helles Design",
            Locale::Es => "Aspecto claro predeterminado",
            Locale::Fr => "Apparence claire par défaut",
            Locale::It => "Aspetto chiaro predefinito",
            Locale::Pl => "Domyślny jasny wygląd",
            Locale::Pt => "Aparência clara padrão",
            Locale::Zh => "默认浅色外观",
            Locale::Ja => "標準のライト外観",
        }
    }

    pub fn theme_dark_desc(&self) -> &'static str {
        match self.locale {
            Locale::En => "High-contrast dark appearance",
            Locale::Cs => "Kontrastní tmavý vzhled",
            Locale::De => "Kontrastreiches dunkles Design",
            Locale::Es => "Aspecto oscuro de alto contraste",
            Locale::Fr => "Apparence sombre contrastée",
            Locale::It => "Aspetto scuro ad alto contrasto",
            Locale::Pl => "Kontrastowy ciemny wygląd",
            Locale::Pt => "Aparência escura de alto contraste",
            Locale::Zh => "高对比度深色外观",
            Locale::Ja => "高コントラストのダーク外観",
        }
    }

    // --- Language Section in Settings ---
    pub fn language_section_title(&self) -> &'static str {
        match self.locale {
            Locale::En => "LANGUAGE",
            Locale::Cs => "JAZYK",
            Locale::De => "SPRACHE",
            Locale::Es => "IDIOMA",
            Locale::Fr => "LANGUE",
            Locale::It => "LINGUA",
            Locale::Pl => "JĘZYK",
            Locale::Pt => "IDIOMA",
            Locale::Zh => "语言",
            Locale::Ja => "言語",
        }
    }

    pub fn language_desc(&self) -> &'static str {
        match self.locale {
            Locale::En => "Choose application interface language",
            Locale::Cs => "Vyberte jazyk uživatelského rozhraní aplikace",
            Locale::De => "Wählen Sie die Sprache der Benutzeroberfläche",
            Locale::Es => "Seleccione el idioma de la interfaz",
            Locale::Fr => "Choisissez la langue de l'interface",
            Locale::It => "Scegli la lingua dell'interfaccia",
            Locale::Pl => "Wybierz język interfejsu aplikacji",
            Locale::Pt => "Escolha o idioma da interface",
            Locale::Zh => "选择应用程序界面语言",
            Locale::Ja => "アプリケーションの表示言語を選択",
        }
    }

    // --- Actions ---
    pub fn act_open(&self) -> &'static str {
        match self.locale {
            Locale::En => "Open",
            Locale::Cs => "Otevřít",
            Locale::De => "Öffnen",
            Locale::Es => "Abrir",
            Locale::Fr => "Ouvrir",
            Locale::It => "Apri",
            Locale::Pl => "Otwórz",
            Locale::Pt => "Abrir",
            Locale::Zh => "打开",
            Locale::Ja => "開く",
        }
    }

    pub fn act_edit(&self) -> &'static str {
        match self.locale {
            Locale::En => "Edit",
            Locale::Cs => "Upravit",
            Locale::De => "Bearbeiten",
            Locale::Es => "Editar",
            Locale::Fr => "Modifier",
            Locale::It => "Modifica",
            Locale::Pl => "Edytuj",
            Locale::Pt => "Editar",
            Locale::Zh => "编辑",
            Locale::Ja => "編集",
        }
    }

    pub fn act_copy(&self) -> &'static str {
        match self.locale {
            Locale::En => "Copy",
            Locale::Cs => "Kopírovat",
            Locale::De => "Kopieren",
            Locale::Es => "Copiar",
            Locale::Fr => "Copier",
            Locale::It => "Copia",
            Locale::Pl => "Kopiuj",
            Locale::Pt => "Copiar",
            Locale::Zh => "复制",
            Locale::Ja => "コピー",
        }
    }

    pub fn act_move(&self) -> &'static str {
        match self.locale {
            Locale::En => "Move",
            Locale::Cs => "Přesunout",
            Locale::De => "Verschieben",
            Locale::Es => "Mover",
            Locale::Fr => "Déplacer",
            Locale::It => "Sposta",
            Locale::Pl => "Przenieś",
            Locale::Pt => "Mover",
            Locale::Zh => "移动",
            Locale::Ja => "移動",
        }
    }

    pub fn act_new_folder(&self) -> &'static str {
        match self.locale {
            Locale::En => "New Folder...",
            Locale::Cs => "Nová složka...",
            Locale::De => "Neuer Ordner...",
            Locale::Es => "Nueva carpeta...",
            Locale::Fr => "Nouveau dossier...",
            Locale::It => "Nuova cartella...",
            Locale::Pl => "Nowy folder...",
            Locale::Pt => "Nova pasta...",
            Locale::Zh => "新建文件夹...",
            Locale::Ja => "新規フォルダ...",
        }
    }

    pub fn act_rename(&self) -> &'static str {
        match self.locale {
            Locale::En => "Rename...",
            Locale::Cs => "Přejmenovat...",
            Locale::De => "Umbenennen...",
            Locale::Es => "Renombrar...",
            Locale::Fr => "Renommer...",
            Locale::It => "Rinomina...",
            Locale::Pl => "Zmień nazwę...",
            Locale::Pt => "Renomear...",
            Locale::Zh => "重命名...",
            Locale::Ja => "名前を変更...",
        }
    }

    pub fn act_delete(&self) -> &'static str {
        match self.locale {
            Locale::En => "Delete...",
            Locale::Cs => "Smazat...",
            Locale::De => "Löschen...",
            Locale::Es => "Eliminar...",
            Locale::Fr => "Supprimer...",
            Locale::It => "Elimina...",
            Locale::Pl => "Usuń...",
            Locale::Pt => "Excluir...",
            Locale::Zh => "删除...",
            Locale::Ja => "削除...",
        }
    }

    pub fn act_permissions(&self) -> &'static str {
        match self.locale {
            Locale::En => "Permissions (CHMOD)...",
            Locale::Cs => "Oprávnění (CHMOD)...",
            Locale::De => "Berechtigungen (CHMOD)...",
            Locale::Es => "Permisos (CHMOD)...",
            Locale::Fr => "Permissions (CHMOD)...",
            Locale::It => "Permessi (CHMOD)...",
            Locale::Pl => "Uprawnienia (CHMOD)...",
            Locale::Pt => "Permissões (CHMOD)...",
            Locale::Zh => "权限 (CHMOD)...",
            Locale::Ja => "アクセス権 (CHMOD)...",
        }
    }

    pub fn act_synchronize(&self) -> &'static str {
        match self.locale {
            Locale::En => "Synchronize Directories...",
            Locale::Cs => "Synchronizovat adresáře...",
            Locale::De => "Verzeichnisse synchronisieren...",
            Locale::Es => "Sincronizar directorios...",
            Locale::Fr => "Synchroniser répertoires...",
            Locale::It => "Sincronizza cartelle...",
            Locale::Pl => "Synchronizuj katalogi...",
            Locale::Pt => "Sincronizar diretórios...",
            Locale::Zh => "同步目录...",
            Locale::Ja => "ディレクトリを同期...",
        }
    }

    pub fn act_reveal_in_finder(&self) -> &'static str {
        match self.locale {
            Locale::En => "Reveal in Finder",
            Locale::Cs => "Zobrazit ve Finderu",
            Locale::De => "Im Finder anzeigen",
            Locale::Es => "Mostrar en Finder",
            Locale::Fr => "Afficher dans le Finder",
            Locale::It => "Mostra nel Finder",
            Locale::Pl => "Pokaż w Finderze",
            Locale::Pt => "Revelar no Finder",
            Locale::Zh => "在访达中显示",
            Locale::Ja => "Finderで表示",
        }
    }

    pub fn act_select_all(&self) -> &'static str {
        match self.locale {
            Locale::En => "Select All",
            Locale::Cs => "Vybrat vše",
            Locale::De => "Alle auswählen",
            Locale::Es => "Seleccionar todo",
            Locale::Fr => "Tout sélectionner",
            Locale::It => "Seleziona tutto",
            Locale::Pl => "Zaznacz wszystko",
            Locale::Pt => "Selecionar tudo",
            Locale::Zh => "全选",
            Locale::Ja => "すべて選択",
        }
    }

    // --- Pane & Table Headers ---
    pub fn local_label(&self) -> &'static str {
        match self.locale {
            Locale::En => "Local",
            Locale::Cs => "Lokální",
            Locale::De => "Lokal",
            Locale::Es => "Local",
            Locale::Fr => "Local",
            Locale::It => "Locale",
            Locale::Pl => "Lokalny",
            Locale::Pt => "Local",
            Locale::Zh => "本地",
            Locale::Ja => "ローカル",
        }
    }

    pub fn col_name(&self) -> &'static str {
        match self.locale {
            Locale::En => "Name",
            Locale::Cs => "Název",
            Locale::De => "Name",
            Locale::Es => "Nombre",
            Locale::Fr => "Nom",
            Locale::It => "Nome",
            Locale::Pl => "Nazwa",
            Locale::Pt => "Nome",
            Locale::Zh => "名称",
            Locale::Ja => "名前",
        }
    }

    pub fn col_size(&self) -> &'static str {
        match self.locale {
            Locale::En => "Size",
            Locale::Cs => "Velikost",
            Locale::De => "Größe",
            Locale::Es => "Tamaño",
            Locale::Fr => "Taille",
            Locale::It => "Dimensione",
            Locale::Pl => "Rozmiar",
            Locale::Pt => "Tamanho",
            Locale::Zh => "大小",
            Locale::Ja => "サイズ",
        }
    }

    pub fn col_modified(&self) -> &'static str {
        match self.locale {
            Locale::En => "Modified",
            Locale::Cs => "Změněno",
            Locale::De => "Geändert",
            Locale::Es => "Modificado",
            Locale::Fr => "Modifié",
            Locale::It => "Modificato",
            Locale::Pl => "Zmodyfikowano",
            Locale::Pt => "Modificado",
            Locale::Zh => "修改时间",
            Locale::Ja => "変更日",
        }
    }

    pub fn col_permissions(&self) -> &'static str {
        match self.locale {
            Locale::En => "Permissions",
            Locale::Cs => "Práva",
            Locale::De => "Rechte",
            Locale::Es => "Permisos",
            Locale::Fr => "Droits",
            Locale::It => "Permessi",
            Locale::Pl => "Uprawnienia",
            Locale::Pt => "Permissões",
            Locale::Zh => "权限",
            Locale::Ja => "権限",
        }
    }

    pub fn folder_type_label(&self) -> &'static str {
        match self.locale {
            Locale::En => "Folder",
            Locale::Cs => "Složka",
            Locale::De => "Ordner",
            Locale::Es => "Carpeta",
            Locale::Fr => "Dossier",
            Locale::It => "Cartella",
            Locale::Pl => "Folder",
            Locale::Pt => "Pasta",
            Locale::Zh => "文件夹",
            Locale::Ja => "フォルダ",
        }
    }

    pub fn status_counts(&self, dirs: usize, files: usize, selected: usize, selected_size: u64) -> String {
        let size_str = crate::model::format_bytes(selected_size);
        match self.locale {
            Locale::En => {
                if selected > 0 {
                    format!("{} folders, {} files ({} selected: {})", dirs, files, selected, size_str)
                } else {
                    format!("{} folders, {} files", dirs, files)
                }
            }
            Locale::Cs => {
                if selected > 0 {
                    format!("{} složek, {} souborů ({} vybráno: {})", dirs, files, selected, size_str)
                } else {
                    format!("{} složek, {} souborů", dirs, files)
                }
            }
            Locale::De => {
                if selected > 0 {
                    format!("{} Ordner, {} Dateien ({} ausgewählt: {})", dirs, files, selected, size_str)
                } else {
                    format!("{} Ordner, {} Dateien", dirs, files)
                }
            }
            Locale::Es => {
                if selected > 0 {
                    format!("{} carpetas, {} archivos ({} seleccionados: {})", dirs, files, selected, size_str)
                } else {
                    format!("{} carpetas, {} archivos", dirs, files)
                }
            }
            Locale::Fr => {
                if selected > 0 {
                    format!("{} dossiers, {} fichiers ({} sélectionnés: {})", dirs, files, selected, size_str)
                } else {
                    format!("{} dossiers, {} fichiers", dirs, files)
                }
            }
            Locale::It => {
                if selected > 0 {
                    format!("{} cartelle, {} file ({} selezionati: {})", dirs, files, selected, size_str)
                } else {
                    format!("{} cartelle, {} file", dirs, files)
                }
            }
            Locale::Pl => {
                if selected > 0 {
                    format!("{} folderów, {} plików ({} zaznaczono: {})", dirs, files, selected, size_str)
                } else {
                    format!("{} folderów, {} plików", dirs, files)
                }
            }
            Locale::Pt => {
                if selected > 0 {
                    format!("{} pastas, {} arquivos ({} selecionados: {})", dirs, files, selected, size_str)
                } else {
                    format!("{} pastas, {} arquivos", dirs, files)
                }
            }
            Locale::Zh => {
                if selected > 0 {
                    format!("{} 个文件夹，{} 个文件（已选 {}: {}）", dirs, files, selected, size_str)
                } else {
                    format!("{} 个文件夹，{} 个文件", dirs, files)
                }
            }
            Locale::Ja => {
                if selected > 0 {
                    format!("{} 個のフォルダ、{} 個のファイル（{} 選択: {}）", dirs, files, selected, size_str)
                } else {
                    format!("{} 個のフォルダ、{} 個のファイル", dirs, files)
                }
            }
        }
    }

    pub fn filter_placeholder(&self) -> &'static str {
        match self.locale {
            Locale::En => "Filter current folder...",
            Locale::Cs => "Filtrovat aktuální složku...",
            Locale::De => "Aktuellen Ordner filtern...",
            Locale::Es => "Filtrar carpeta actual...",
            Locale::Fr => "Filtrer le dossier actuel...",
            Locale::It => "Filtra cartella corrente...",
            Locale::Pl => "Filtruj bieżący katalog...",
            Locale::Pt => "Filtrar pasta atual...",
            Locale::Zh => "过滤当前文件夹...",
            Locale::Ja => "現在のフォルダを検索...",
        }
    }

    // --- Transfer Queue ---
    pub fn transfer_queue_title(&self) -> &'static str {
        match self.locale {
            Locale::En => "Transfer Queue",
            Locale::Cs => "Fronta přenosů",
            Locale::De => "Warteschlange",
            Locale::Es => "Cola de transferencias",
            Locale::Fr => "File d'attente",
            Locale::It => "Coda trasferimenti",
            Locale::Pl => "Kolejka transferu",
            Locale::Pt => "Fila de transferências",
            Locale::Zh => "传输队列",
            Locale::Ja => "転送キュー",
        }
    }

    pub fn transfer_uploading(&self) -> &'static str {
        match self.locale {
            Locale::En => "Uploading",
            Locale::Cs => "Nahrávání",
            Locale::De => "Hochladen",
            Locale::Es => "Subiendo",
            Locale::Fr => "Téléversement",
            Locale::It => "Caricamento",
            Locale::Pl => "Wysyłanie",
            Locale::Pt => "Enviando",
            Locale::Zh => "上传中",
            Locale::Ja => "アップロード中",
        }
    }

    pub fn transfer_downloading(&self) -> &'static str {
        match self.locale {
            Locale::En => "Downloading",
            Locale::Cs => "Stahování",
            Locale::De => "Herunterladen",
            Locale::Es => "Descargando",
            Locale::Fr => "Téléchargement",
            Locale::It => "Scaricamento",
            Locale::Pl => "Pobieranie",
            Locale::Pt => "Baixando",
            Locale::Zh => "下载中",
            Locale::Ja => "ダウンロード中",
        }
    }

    pub fn transfer_completed(&self) -> &'static str {
        match self.locale {
            Locale::En => "Completed",
            Locale::Cs => "Dokončeno",
            Locale::De => "Abgeschlossen",
            Locale::Es => "Completado",
            Locale::Fr => "Terminé",
            Locale::It => "Completato",
            Locale::Pl => "Ukończono",
            Locale::Pt => "Concluído",
            Locale::Zh => "已完成",
            Locale::Ja => "完了",
        }
    }

    pub fn transfer_queued(&self) -> &'static str {
        match self.locale {
            Locale::En => "Queued",
            Locale::Cs => "Ve frontě",
            Locale::De => "In Warteschlange",
            Locale::Es => "En cola",
            Locale::Fr => "En attente",
            Locale::It => "In coda",
            Locale::Pl => "W kolejce",
            Locale::Pt => "Na fila",
            Locale::Zh => "等待中",
            Locale::Ja => "待機中",
        }
    }

    // --- Modals & Buttons ---
    pub fn modal_site_manager_title(&self) -> &'static str {
        match self.locale {
            Locale::En => "Connection Manager",
            Locale::Cs => "Správce spojení",
            Locale::De => "Verbindungsmanager",
            Locale::Es => "Gestor de conexiones",
            Locale::Fr => "Gestionnaire de connexions",
            Locale::It => "Gestione connessioni",
            Locale::Pl => "Menedżer połączeń",
            Locale::Pt => "Gerenciador de conexões",
            Locale::Zh => "站点管理器",
            Locale::Ja => "接続マネージャー",
        }
    }

    pub fn label_protocol(&self) -> &'static str {
        match self.locale {
            Locale::En => "Protocol",
            Locale::Cs => "Protokol",
            Locale::De => "Protokoll",
            Locale::Es => "Protocolo",
            Locale::Fr => "Protocole",
            Locale::It => "Protocollo",
            Locale::Pl => "Protokół",
            Locale::Pt => "Protocolo",
            Locale::Zh => "协议",
            Locale::Ja => "プロトコル",
        }
    }

    pub fn label_host(&self) -> &'static str {
        match self.locale {
            Locale::En => "Host:",
            Locale::Cs => "Hostitel:",
            Locale::De => "Host:",
            Locale::Es => "Servidor:",
            Locale::Fr => "Hôte:",
            Locale::It => "Host:",
            Locale::Pl => "Host:",
            Locale::Pt => "Host:",
            Locale::Zh => "主机:",
            Locale::Ja => "ホスト:",
        }
    }

    pub fn label_port(&self) -> &'static str {
        match self.locale {
            Locale::En => "Port:",
            Locale::Cs => "Port:",
            Locale::De => "Port:",
            Locale::Es => "Puerto:",
            Locale::Fr => "Port:",
            Locale::It => "Porta:",
            Locale::Pl => "Port:",
            Locale::Pt => "Porta:",
            Locale::Zh => "端口:",
            Locale::Ja => "ポート:",
        }
    }

    pub fn label_user(&self) -> &'static str {
        match self.locale {
            Locale::En => "User:",
            Locale::Cs => "Uživatel:",
            Locale::De => "Benutzer:",
            Locale::Es => "Usuario:",
            Locale::Fr => "Utilisateur:",
            Locale::It => "Utente:",
            Locale::Pl => "Użytkownik:",
            Locale::Pt => "Usuário:",
            Locale::Zh => "用户:",
            Locale::Ja => "ユーザー:",
        }
    }

    pub fn label_password(&self) -> &'static str {
        match self.locale {
            Locale::En => "Password:",
            Locale::Cs => "Heslo:",
            Locale::De => "Passwort:",
            Locale::Es => "Contraseña:",
            Locale::Fr => "Mot de passe:",
            Locale::It => "Password:",
            Locale::Pl => "Hasło:",
            Locale::Pt => "Senha:",
            Locale::Zh => "密码:",
            Locale::Ja => "パスワード:",
        }
    }

    pub fn label_max_connections(&self) -> &'static str {
        match self.locale {
            Locale::En => "Max Connections:",
            Locale::Cs => "Max. spojení:",
            Locale::De => "Max. Verbindungen:",
            Locale::Es => "Conexiones máx.:",
            Locale::Fr => "Connexions max.:",
            Locale::It => "Connessioni massime:",
            Locale::Pl => "Maks. połączeń:",
            Locale::Pt => "Conexões máx.:",
            Locale::Zh => "最大连接数:",
            Locale::Ja => "最大接続数:",
        }
    }

    pub fn transfers_section_title(&self) -> &'static str {
        match self.locale {
            Locale::En => "TRANSFERS & CONCURRENT CONNECTIONS",
            Locale::Cs => "PŘENOSY A PARALELNÍ SPOJENÍ",
            Locale::De => "ÜBERTRAGUNGEN & PARALLELE VERBINDUNGEN",
            Locale::Es => "TRANSFERENCIAS Y CONEXIONES SIMULTÁNEAS",
            Locale::Fr => "TRANSFERTS ET CONNEXIONS SIMULTANÉES",
            Locale::It => "TRASFERIMENTI E CONNESSIONI SIMULTANEE",
            Locale::Pl => "TRANSFERY I POŁĄCZENIA RÓWNOLEGŁE",
            Locale::Pt => "TRANSFERÊNCIAS E CONEXÕES SIMULTÂNEAS",
            Locale::Zh => "传输与并发连接",
            Locale::Ja => "転送と並行接続",
        }
    }

    pub fn max_concurrent_desc(&self) -> &'static str {
        match self.locale {
            Locale::En => "Default number of parallel file transfers (1–10)",
            Locale::Cs => "Výchozí počet paralelních přenosů souborů (1–10)",
            Locale::De => "Standardanzahl paralleler Dateiübertragungen (1–10)",
            Locale::Es => "Número predeterminado de transferencias de archivos en paralelo (1–10)",
            Locale::Fr => "Nombre par défaut de transferts de fichiers en parallèle (1–10)",
            Locale::It => "Numero predefinito di trasferimenti file paralleli (1–10)",
            Locale::Pl => "Domyślna liczba równoległych transferów plików (1–10)",
            Locale::Pt => "Número padrão de transferências paralelas de arquivos (1–10)",
            Locale::Zh => "默认并发文件传输数 (1–10)",
            Locale::Ja => "デフォルトの並行ファイル転送数 (1–10)",
        }
    }

    pub fn label_environment(&self) -> &'static str {
        match self.locale {
            Locale::En => "Color:",
            Locale::Cs => "Barva:",
            Locale::De => "Farbe:",
            Locale::Es => "Color:",
            Locale::Fr => "Couleur:",
            Locale::It => "Colore:",
            Locale::Pl => "Kolor:",
            Locale::Pt => "Cor:",
            Locale::Zh => "颜色:",
            Locale::Ja => "カラー:",
        }
    }

    pub fn environment_name(&self, color: u32) -> &'static str {
        match color {
            0xef4444 | 0xdc2626 => match self.locale {
                Locale::En => "Red",
                Locale::Cs => "Červená",
                Locale::De => "Rot",
                Locale::Es => "Rojo",
                Locale::Fr => "Rouge",
                Locale::It => "Rosso",
                Locale::Pl => "Czerwony",
                Locale::Pt => "Vermelho",
                Locale::Zh => "红色",
                Locale::Ja => "赤",
            },
            0xf59e0b | 0xea580c => match self.locale {
                Locale::En => "Orange",
                Locale::Cs => "Oranžová",
                Locale::De => "Orange",
                Locale::Es => "Naranja",
                Locale::Fr => "Orange",
                Locale::It => "Arancione",
                Locale::Pl => "Pomarańczowy",
                Locale::Pt => "Laranja",
                Locale::Zh => "橙色",
                Locale::Ja => "オレンジ",
            },
            0x2563eb | 0x3b82f6 => match self.locale {
                Locale::En => "Blue",
                Locale::Cs => "Modrá",
                Locale::De => "Blau",
                Locale::Es => "Azul",
                Locale::Fr => "Bleu",
                Locale::It => "Blu",
                Locale::Pl => "Niebieski",
                Locale::Pt => "Azul",
                Locale::Zh => "蓝色",
                Locale::Ja => "青",
            },
            0x10b981 | 0x059669 | 0x16a34a => match self.locale {
                Locale::En => "Green",
                Locale::Cs => "Zelená",
                Locale::De => "Grün",
                Locale::Es => "Verde",
                Locale::Fr => "Vert",
                Locale::It => "Verde",
                Locale::Pl => "Zielony",
                Locale::Pt => "Verde",
                Locale::Zh => "绿色",
                Locale::Ja => "緑",
            },
            0x8b5cf6 | 0x7c3aed => match self.locale {
                Locale::En => "Purple",
                Locale::Cs => "Fialová",
                Locale::De => "Lila",
                Locale::Es => "Morado",
                Locale::Fr => "Violet",
                Locale::It => "Viola",
                Locale::Pl => "Fioletowy",
                Locale::Pt => "Roxo",
                Locale::Zh => "紫色",
                Locale::Ja => "紫",
            },
            _ => match self.locale {
                Locale::En => "Custom",
                Locale::Cs => "Vlastní",
                Locale::De => "Benutzerdefiniert",
                Locale::Es => "Personalizado",
                Locale::Fr => "Personnalisé",
                Locale::It => "Personalizzato",
                Locale::Pl => "Niestandardowy",
                Locale::Pt => "Personalizado",
                Locale::Zh => "自定义",
                Locale::Ja => "カスタム",
            },
        }
    }

    pub fn btn_connect(&self) -> &'static str {
        match self.locale {
            Locale::En => "Connect",
            Locale::Cs => "Připojit",
            Locale::De => "Verbinden",
            Locale::Es => "Conectar",
            Locale::Fr => "Connexion",
            Locale::It => "Connetti",
            Locale::Pl => "Połącz",
            Locale::Pt => "Conectar",
            Locale::Zh => "连接",
            Locale::Ja => "接続",
        }
    }

    pub fn btn_connecting(&self) -> &'static str {
        match self.locale {
            Locale::En => "Connecting...",
            Locale::Cs => "Připojuji...",
            Locale::De => "Verbinde...",
            Locale::Es => "Conectando...",
            Locale::Fr => "Connexion...",
            Locale::It => "Connessione...",
            Locale::Pl => "Łączenie...",
            Locale::Pt => "Conectando...",
            Locale::Zh => "正在连接...",
            Locale::Ja => "接続中...",
        }
    }

    pub fn site_connecting_status(&self, host: &str, port: u16) -> String {
        match self.locale {
            Locale::En => format!("Connecting to {}:{}...", host, port),
            Locale::Cs => format!("Připojování k {}:{}...", host, port),
            Locale::De => format!("Verbindungsaufbau zu {}:{}...", host, port),
            Locale::Es => format!("Conectando a {}:{}...", host, port),
            Locale::Fr => format!("Connexion à {}:{}...", host, port),
            Locale::It => format!("Connessione a {}:{}...", host, port),
            Locale::Pl => format!("Nawiązywanie połączenia z {}:{}...", host, port),
            Locale::Pt => format!("Conectando a {}:{}...", host, port),
            Locale::Zh => format!("正在连接到 {}:{}...", host, port),
            Locale::Ja => format!("{}:{} に接続中...", host, port),
        }
    }

    pub fn btn_cancel(&self) -> &'static str {
        match self.locale {
            Locale::En => "Cancel",
            Locale::Cs => "Zrušit",
            Locale::De => "Abbrechen",
            Locale::Es => "Cancelar",
            Locale::Fr => "Annuler",
            Locale::It => "Annulla",
            Locale::Pl => "Anuluj",
            Locale::Pt => "Cancelar",
            Locale::Zh => "取消",
            Locale::Ja => "キャンセル",
        }
    }

    pub fn btn_save(&self) -> &'static str {
        match self.locale {
            Locale::En => "Save",
            Locale::Cs => "Uložit",
            Locale::De => "Speichern",
            Locale::Es => "Guardar",
            Locale::Fr => "Enregistrer",
            Locale::It => "Salva",
            Locale::Pl => "Zapisz",
            Locale::Pt => "Salvar",
            Locale::Zh => "保存",
            Locale::Ja => "保存",
        }
    }

    pub fn btn_create(&self) -> &'static str {
        match self.locale {
            Locale::En => "Create",
            Locale::Cs => "Vytvořit",
            Locale::De => "Erstellen",
            Locale::Es => "Crear",
            Locale::Fr => "Créer",
            Locale::It => "Crea",
            Locale::Pl => "Utwórz",
            Locale::Pt => "Criar",
            Locale::Zh => "创建",
            Locale::Ja => "作成",
        }
    }

    pub fn btn_delete(&self) -> &'static str {
        match self.locale {
            Locale::En => "Delete",
            Locale::Cs => "Smazat",
            Locale::De => "Löschen",
            Locale::Es => "Eliminar",
            Locale::Fr => "Supprimer",
            Locale::It => "Elimina",
            Locale::Pl => "Usuń",
            Locale::Pt => "Excluir",
            Locale::Zh => "删除",
            Locale::Ja => "削除",
        }
    }

    pub fn btn_run(&self) -> &'static str {
        match self.locale {
            Locale::En => "Start",
            Locale::Cs => "Spustit",
            Locale::De => "Starten",
            Locale::Es => "Iniciar",
            Locale::Fr => "Démarrer",
            Locale::It => "Avvia",
            Locale::Pl => "Start",
            Locale::Pt => "Iniciar",
            Locale::Zh => "开始",
            Locale::Ja => "開始",
        }
    }

    pub fn btn_start(&self) -> &'static str {
        self.btn_run()
    }

    pub fn btn_import_ssh(&self) -> &'static str {
        match self.locale {
            Locale::En => "Import SSH",
            Locale::Cs => "Importovat SSH",
            Locale::De => "SSH importieren",
            Locale::Es => "Importar SSH",
            Locale::Fr => "Importer SSH",
            Locale::It => "Importa SSH",
            Locale::Pl => "Importuj SSH",
            Locale::Pt => "Importar SSH",
            Locale::Zh => "导入 SSH",
            Locale::Ja => "SSHインポート",
        }
    }

    pub fn notice_ssh_config_not_found(&self) -> &'static str {
        match self.locale {
            Locale::En => "File ~/.ssh/config was not found.",
            Locale::Cs => "Soubor ~/.ssh/config nebyl nalezen.",
            Locale::De => "Datei ~/.ssh/config wurde nicht gefunden.",
            Locale::Es => "No se encontró el archivo ~/.ssh/config.",
            Locale::Fr => "Fichier ~/.ssh/config introuvable.",
            Locale::It => "File ~/.ssh/config non trovato.",
            Locale::Pl => "Nie znaleziono pliku ~/.ssh/config.",
            Locale::Pt => "Arquivo ~/.ssh/config não encontrado.",
            Locale::Zh => "未找到 ~/.ssh/config 文件。",
            Locale::Ja => "~/.ssh/config ファイルが見つかりません。",
        }
    }

    pub fn notice_ssh_config_no_hosts(&self) -> &'static str {
        match self.locale {
            Locale::En => "No host configurations found in ~/.ssh/config.",
            Locale::Cs => "V ~/.ssh/config nebyly nalezeny žádné servery.",
            Locale::De => "Keine Host-Konfigurationen in ~/.ssh/config gefunden.",
            Locale::Es => "No se encontraron configuraciones de host en ~/.ssh/config.",
            Locale::Fr => "Aucune configuration d'hôte trouvée dans ~/.ssh/config.",
            Locale::It => "Nessuna configurazione host trovata in ~/.ssh/config.",
            Locale::Pl => "Nie znaleziono konfiguracji hostów w ~/.ssh/config.",
            Locale::Pt => "Nenhuma configuração de host encontrada em ~/.ssh/config.",
            Locale::Zh => "在 ~/.ssh/config 中未找到有效的主机配置。",
            Locale::Ja => "~/.ssh/config にホスト設定が見つかりませんでした。",
        }
    }

    pub fn notice_ssh_config_imported(&self, count: usize) -> String {
        match self.locale {
            Locale::En => format!(
                "Successfully imported {} server{} from ~/.ssh/config",
                count,
                if count == 1 { "" } else { "s" }
            ),
            Locale::Cs => {
                if count == 1 {
                    "Úspěšně importován 1 server z ~/.ssh/config".to_string()
                } else if count < 5 {
                    format!("Úspěšně importovány {} servery z ~/.ssh/config", count)
                } else {
                    format!("Úspěšně importováno {} serverů z ~/.ssh/config", count)
                }
            }
            Locale::De => format!("{} Server erfolgreich aus ~/.ssh/config importiert", count),
            Locale::Es => format!(
                "{} servidor(es) importado(s) exitosamente desde ~/.ssh/config",
                count
            ),
            Locale::Fr => format!(
                "{} serveur(s) importé(s) avec succès depuis ~/.ssh/config",
                count
            ),
            Locale::It => format!("{} server importati con successo da ~/.ssh/config", count),
            Locale::Pl => {
                if count == 1 {
                    "Pomyślnie zaimportowano 1 serwer z ~/.ssh/config".to_string()
                } else if count < 5 {
                    format!("Pomyślnie zaimportowano {} serwery z ~/.ssh/config", count)
                } else {
                    format!("Pomyślnie zaimportowano {} serwerów z ~/.ssh/config", count)
                }
            }
            Locale::Pt => format!(
                "{} servidor(es) importado(s) com sucesso de ~/.ssh/config",
                count
            ),
            Locale::Zh => format!("成功从 ~/.ssh/config 导入 {} 个服务器", count),
            Locale::Ja => {
                format!("~/.ssh/config から {} 件のサーバーを正常にインポートしました", count)
            }
        }
    }

    pub fn notice_ssh_config_all_exist(&self, count: usize) -> String {
        match self.locale {
            Locale::En => format!(
                "All {} server{} from ~/.ssh/config already exist in your list",
                count,
                if count == 1 { "" } else { "s" }
            ),
            Locale::Cs => {
                if count == 1 {
                    "Server z ~/.ssh/config již máte v seznamu".to_string()
                } else {
                    format!("Všech {} serverů z ~/.ssh/config již máte v seznamu", count)
                }
            }
            Locale::De => format!(
                "Alle {} Server aus ~/.ssh/config sind bereits in Ihrer Liste",
                count
            ),
            Locale::Es => format!(
                "Todos los {} servidor(es) de ~/.ssh/config ya están en su lista",
                count
            ),
            Locale::Fr => format!(
                "Les {} serveur(s) de ~/.ssh/config sont déjà dans votre liste",
                count
            ),
            Locale::It => format!(
                "Tutti i {} server da ~/.ssh/config sono già nella tua lista",
                count
            ),
            Locale::Pl => format!(
                "Wszystkie {} serwery z ~/.ssh/config znajdują się już na Twojej liście",
                count
            ),
            Locale::Pt => format!(
                "Todos os {} servidor(es) de ~/.ssh/config já estão na sua lista",
                count
            ),
            Locale::Zh => format!("~/.ssh/config 中的所有 {} 个服务器已存在于您的列表中", count),
            Locale::Ja => format!(
                "~/.ssh/config の全 {} 件のサーバーはすでにリストに存在します",
                count
            ),
        }
    }

    pub fn modal_chmod_title(&self) -> &'static str {
        match self.locale {
            Locale::En => "Permissions (CHMOD)",
            Locale::Cs => "Oprávnění (CHMOD)",
            Locale::De => "Berechtigungen (CHMOD)",
            Locale::Es => "Permisos (CHMOD)",
            Locale::Fr => "Permissions (CHMOD)",
            Locale::It => "Permessi (CHMOD)",
            Locale::Pl => "Uprawnienia (CHMOD)",
            Locale::Pt => "Permissões (CHMOD)",
            Locale::Zh => "权限 (CHMOD)",
            Locale::Ja => "アクセス権 (CHMOD)",
        }
    }

    pub fn chmod_recursive_label(&self, enabled: bool) -> &'static str {
        match (self.locale, enabled) {
            (Locale::En, true) => "[x] Apply recursively to subdirectories",
            (Locale::En, false) => "[ ] Apply recursively to subdirectories",
            (Locale::Cs, true) => "[x] Aplikovat rekurzivně na podsložky",
            (Locale::Cs, false) => "[ ] Aplikovat rekurzivně na podsložky",
            (Locale::De, true) => "[x] Rekursiv auf Unterordner anwenden",
            (Locale::De, false) => "[ ] Rekursiv auf Unterordner anwenden",
            (Locale::Es, true) => "[x] Aplicar recursivamente a subdirectorios",
            (Locale::Es, false) => "[ ] Aplicar recursivamente a subdirectorios",
            (Locale::Fr, true) => "[x] Appliquer récursivement aux sous-dossiers",
            (Locale::Fr, false) => "[ ] Appliquer récursivement aux sous-dossiers",
            (Locale::It, true) => "[x] Applica ricorsivamente alle sottocartelle",
            (Locale::It, false) => "[ ] Applica ricorsivamente alle sottocartelle",
            (Locale::Pl, true) => "[x] Zastosuj rekurencyjnie do podfolderów",
            (Locale::Pl, false) => "[ ] Zastosuj rekurencyjnie do podfolderów",
            (Locale::Pt, true) => "[x] Aplicar recursivamente a subpastas",
            (Locale::Pt, false) => "[ ] Aplicar recursivamente a subpastas",
            (Locale::Zh, true) => "[x] 递归应用到子目录",
            (Locale::Zh, false) => "[ ] 递归应用到子目录",
            (Locale::Ja, true) => "[x] サブディレクトリにも再帰的に適用",
            (Locale::Ja, false) => "[ ] サブディレクトリにも再帰的に適用",
        }
    }

    pub fn chmod_enabled(&self, on: bool) -> &'static str {
        match (self.locale, on) {
            (Locale::En, true) => "Enabled",
            (Locale::En, false) => "Disabled",
            (Locale::Cs, true) => "Zapnuto",
            (Locale::Cs, false) => "Vypnuto",
            (Locale::De, true) => "Aktiviert",
            (Locale::De, false) => "Deaktiviert",
            (Locale::Es, true) => "Activado",
            (Locale::Es, false) => "Desactivado",
            (Locale::Fr, true) => "Activé",
            (Locale::Fr, false) => "Désactivé",
            (Locale::It, true) => "Attivo",
            (Locale::It, false) => "Disattivo",
            (Locale::Pl, true) => "Włączone",
            (Locale::Pl, false) => "Wyłączone",
            (Locale::Pt, true) => "Ativado",
            (Locale::Pt, false) => "Desativado",
            (Locale::Zh, true) => "已启用",
            (Locale::Zh, false) => "已禁用",
            (Locale::Ja, true) => "有効",
            (Locale::Ja, false) => "無効",
        }
    }

    pub fn modal_sync_title(&self) -> &'static str {
        match self.locale {
            Locale::En => "Synchronize Directories",
            Locale::Cs => "Synchronizace adresářů",
            Locale::De => "Verzeichnisse synchronisieren",
            Locale::Es => "Sincronizar directorios",
            Locale::Fr => "Synchroniser répertoires",
            Locale::It => "Sincronizza cartelle",
            Locale::Pl => "Synchronizuj katalogi",
            Locale::Pt => "Sincronizar diretórios",
            Locale::Zh => "同步目录",
            Locale::Ja => "ディレクトリを同期",
        }
    }

    pub fn sync_dir_local_to_remote(&self) -> &'static str {
        match self.locale {
            Locale::En => "Local ➔ Remote (Upload)",
            Locale::Cs => "Lokální ➔ Vzdálený (Nahrát)",
            Locale::De => "Lokal ➔ Server (Hochladen)",
            Locale::Es => "Local ➔ Remoto (Subir)",
            Locale::Fr => "Local ➔ Distant (Téléverser)",
            Locale::It => "Locale ➔ Remoto (Carica)",
            Locale::Pl => "Lokalny ➔ Zdalny (Wyślij)",
            Locale::Pt => "Local ➔ Remoto (Enviar)",
            Locale::Zh => "本地 ➔ 远程 (上传)",
            Locale::Ja => "ローカル ➔ リモート (アップロード)",
        }
    }

    pub fn sync_dir_remote_to_local(&self) -> &'static str {
        match self.locale {
            Locale::En => "Remote ➔ Local (Download)",
            Locale::Cs => "Vzdálený ➔ Lokální (Stáhnout)",
            Locale::De => "Server ➔ Lokal (Herunterladen)",
            Locale::Es => "Remoto ➔ Local (Descargar)",
            Locale::Fr => "Distant ➔ Local (Télécharger)",
            Locale::It => "Remoto ➔ Locale (Scarica)",
            Locale::Pl => "Zdalny ➔ Lokalny (Pobierz)",
            Locale::Pt => "Remoto ➔ Local (Baixar)",
            Locale::Zh => "远程 ➔ 本地 (下载)",
            Locale::Ja => "リモート ➔ ローカル (ダウンロード)",
        }
    }

    pub fn sync_dir_two_way(&self) -> &'static str {
        match self.locale {
            Locale::En => "Both Directions ⇄",
            Locale::Cs => "Obousměrně ⇄",
            Locale::De => "Beide Richtungen ⇄",
            Locale::Es => "Ambas direcciones ⇄",
            Locale::Fr => "Bidirectionnel ⇄",
            Locale::It => "Entrambe le direzioni ⇄",
            Locale::Pl => "Oba kierunki ⇄",
            Locale::Pt => "Ambas as direções ⇄",
            Locale::Zh => "双向同步 ⇄",
            Locale::Ja => "双方向同期 ⇄",
        }
    }

    pub fn sync_opt_recursive(&self) -> &'static str {
        match self.locale {
            Locale::En => "Include subfolders",
            Locale::Cs => "Rekurzivně (podsložky)",
            Locale::De => "Unterordner einbeziehen",
            Locale::Es => "Incluir subcarpetas",
            Locale::Fr => "Inclure sous-dossiers",
            Locale::It => "Includi sottocartelle",
            Locale::Pl => "Uwzględnij podkatalogi",
            Locale::Pt => "Incluir subpastas",
            Locale::Zh => "包含子文件夹",
            Locale::Ja => "サブフォルダを含める",
        }
    }

    pub fn sync_opt_time_size(&self) -> &'static str {
        match self.locale {
            Locale::En => "Modtime & size",
            Locale::Cs => "Čas úpravy a velikost",
            Locale::De => "Zeit & Größe",
            Locale::Es => "Fecha y tamaño",
            Locale::Fr => "Date et taille",
            Locale::It => "Data e dimensione",
            Locale::Pl => "Czas i rozmiar",
            Locale::Pt => "Data e tamanho",
            Locale::Zh => "修改时间和大小",
            Locale::Ja => "更新日時とサイズ",
        }
    }

    pub fn sync_opt_mirror(&self) -> &'static str {
        match self.locale {
            Locale::En => "Mirror (delete missing in target)",
            Locale::Cs => "Zrcadlení (smazat chybějící v cíli)",
            Locale::De => "Spiegeln (fehlende im Ziel löschen)",
            Locale::Es => "Espejo (eliminar faltantes en destino)",
            Locale::Fr => "Miroir (supprimer absents de la cible)",
            Locale::It => "Mirror (elimina mancanti in destinazione)",
            Locale::Pl => "Lustro (usuń brakujące w celu)",
            Locale::Pt => "Espelho (excluir ausentes no destino)",
            Locale::Zh => "镜像 (删除目标中缺失项)",
            Locale::Ja => "ミラー (ターゲットの欠落ファイルを削除)",
        }
    }

    pub fn sync_btn_compare(&self) -> &'static str {
        match self.locale {
            Locale::En => "Compare Directories",
            Locale::Cs => "Porovnat adresáře",
            Locale::De => "Verzeichnisse vergleichen",
            Locale::Es => "Comparar directorios",
            Locale::Fr => "Comparer répertoires",
            Locale::It => "Confronta cartelle",
            Locale::Pl => "Porównaj katalogi",
            Locale::Pt => "Comparar diretórios",
            Locale::Zh => "比较目录",
            Locale::Ja => "ディレクトリを比較",
        }
    }

    pub fn sync_btn_start(&self) -> &'static str {
        match self.locale {
            Locale::En => "Start Synchronization",
            Locale::Cs => "Spustit synchronizaci",
            Locale::De => "Synchronisation starten",
            Locale::Es => "Iniciar sincronización",
            Locale::Fr => "Démarrer synchronisation",
            Locale::It => "Avvia sincronizzazione",
            Locale::Pl => "Rozpocznij synchronizację",
            Locale::Pt => "Iniciar sincronização",
            Locale::Zh => "开始同步",
            Locale::Ja => "同期を開始",
        }
    }

    pub fn sync_status_comparing(&self) -> &'static str {
        match self.locale {
            Locale::En => "Comparing directories...",
            Locale::Cs => "Porovnávám adresáře...",
            Locale::De => "Verzeichnisse werden verglichen...",
            Locale::Es => "Comparando directorios...",
            Locale::Fr => "Comparaison des répertoires...",
            Locale::It => "Confronto in corso...",
            Locale::Pl => "Porównywanie katalogów...",
            Locale::Pt => "Comparando diretórios...",
            Locale::Zh => "正在比较目录...",
            Locale::Ja => "ディレクトリを比較中...",
        }
    }

    pub fn sync_status_identical(&self) -> &'static str {
        match self.locale {
            Locale::En => "Directories are synchronized. No differences found.",
            Locale::Cs => "Adresáře jsou synchronizované. Žádné rozdíly nebyly nalezeny.",
            Locale::De => "Verzeichnisse sind synchronisiert. Keine Unterschiede.",
            Locale::Es => "Directorios sincronizados. No se encontraron diferencias.",
            Locale::Fr => "Répertoires synchronisés. Aucune différence trouvée.",
            Locale::It => "Le cartelle sono sincronizzate. Nessuna differenza.",
            Locale::Pl => "Katalogi są zsynchronizowane. Brak różnic.",
            Locale::Pt => "Diretórios sincronizados. Nenhuma diferença encontrada.",
            Locale::Zh => "目录已同步，未发现任何差异。",
            Locale::Ja => "ディレクトリは同期されています。差分はありません。",
        }
    }

    pub fn sync_col_file(&self) -> &'static str {
        match self.locale {
            Locale::En => "File / Path",
            Locale::Cs => "Soubor / Cesta",
            Locale::De => "Datei / Pfad",
            Locale::Es => "Archivo / Ruta",
            Locale::Fr => "Fichier / Chemin",
            Locale::It => "File / Percorso",
            Locale::Pl => "Plik / Ścieżka",
            Locale::Pt => "Arquivo / Caminho",
            Locale::Zh => "文件 / 路径",
            Locale::Ja => "ファイル / パス",
        }
    }

    pub fn sync_col_action(&self) -> &'static str {
        match self.locale {
            Locale::En => "Action",
            Locale::Cs => "Akce",
            Locale::De => "Aktion",
            Locale::Es => "Acción",
            Locale::Fr => "Action",
            Locale::It => "Azione",
            Locale::Pl => "Akcja",
            Locale::Pt => "Ação",
            Locale::Zh => "操作",
            Locale::Ja => "アクション",
        }
    }

    pub fn sync_col_source(&self) -> &'static str {
        match self.locale {
            Locale::En => "Local",
            Locale::Cs => "Lokální",
            Locale::De => "Lokal",
            Locale::Es => "Local",
            Locale::Fr => "Local",
            Locale::It => "Locale",
            Locale::Pl => "Lokalny",
            Locale::Pt => "Local",
            Locale::Zh => "本地",
            Locale::Ja => "ローカル",
        }
    }

    pub fn sync_col_target(&self) -> &'static str {
        match self.locale {
            Locale::En => "Remote",
            Locale::Cs => "Vzdálený",
            Locale::De => "Server",
            Locale::Es => "Remoto",
            Locale::Fr => "Distant",
            Locale::It => "Remoto",
            Locale::Pl => "Zdalny",
            Locale::Pt => "Remoto",
            Locale::Zh => "远程",
            Locale::Ja => "リモート",
        }
    }

    pub fn sync_col_reason(&self) -> &'static str {
        match self.locale {
            Locale::En => "Reason",
            Locale::Cs => "Důvod",
            Locale::De => "Grund",
            Locale::Es => "Motivo",
            Locale::Fr => "Raison",
            Locale::It => "Motivo",
            Locale::Pl => "Powód",
            Locale::Pt => "Motivo",
            Locale::Zh => "原因",
            Locale::Ja => "理由",
        }
    }

    pub fn sync_select_all(&self) -> &'static str {
        match self.locale {
            Locale::En => "Select All",
            Locale::Cs => "Vybrat vše",
            Locale::De => "Alle auswählen",
            Locale::Es => "Seleccionar todo",
            Locale::Fr => "Tout sélectionner",
            Locale::It => "Seleziona tutto",
            Locale::Pl => "Zaznacz wszystko",
            Locale::Pt => "Selecionar tudo",
            Locale::Zh => "全选",
            Locale::Ja => "すべて選択",
        }
    }

    pub fn sync_deselect_all(&self) -> &'static str {
        match self.locale {
            Locale::En => "Deselect All",
            Locale::Cs => "Odznačit vše",
            Locale::De => "Alle abwählen",
            Locale::Es => "Deseleccionar todo",
            Locale::Fr => "Tout désélectionner",
            Locale::It => "Deseleziona tutto",
            Locale::Pl => "Odznacz wszystko",
            Locale::Pt => "Desmarcar tudo",
            Locale::Zh => "取消全选",
            Locale::Ja => "選択解除",
        }
    }

    pub fn modal_new_folder_title(&self) -> &'static str {
        match self.locale {
            Locale::En => "New Folder",
            Locale::Cs => "Nová složka",
            Locale::De => "Neuer Ordner",
            Locale::Es => "Nueva carpeta",
            Locale::Fr => "Nouveau dossier",
            Locale::It => "Nuova cartella",
            Locale::Pl => "Nowy folder",
            Locale::Pt => "Nova pasta",
            Locale::Zh => "新建文件夹",
            Locale::Ja => "新規フォルダ",
        }
    }

    pub fn modal_rename_title(&self) -> &'static str {
        match self.locale {
            Locale::En => "Rename",
            Locale::Cs => "Přejmenovat",
            Locale::De => "Umbenennen",
            Locale::Es => "Renombrar",
            Locale::Fr => "Renommer",
            Locale::It => "Rinomina",
            Locale::Pl => "Zmień nazwę",
            Locale::Pt => "Renomear",
            Locale::Zh => "重命名",
            Locale::Ja => "名前を変更",
        }
    }

    pub fn modal_delete_title(&self) -> &'static str {
        match self.locale {
            Locale::En => "Delete Item",
            Locale::Cs => "Smazat položku",
            Locale::De => "Element löschen",
            Locale::Es => "Eliminar elemento",
            Locale::Fr => "Supprimer l'élément",
            Locale::It => "Elimina elemento",
            Locale::Pl => "Usuń element",
            Locale::Pt => "Excluir item",
            Locale::Zh => "删除项目",
            Locale::Ja => "項目を削除",
        }
    }

    pub fn modal_go_to_folder_title(&self) -> &'static str {
        match self.locale {
            Locale::En => "Go to Folder",
            Locale::Cs => "Přejít do složky",
            Locale::De => "Gehe zu Ordner",
            Locale::Es => "Ir a la carpeta",
            Locale::Fr => "Aller au dossier",
            Locale::It => "Vai alla cartella",
            Locale::Pl => "Przejdź do folderu",
            Locale::Pt => "Ir para a pasta",
            Locale::Zh => "前往文件夹",
            Locale::Ja => "フォルダへ移動",
        }
    }

    pub fn btn_go(&self) -> &'static str {
        match self.locale {
            Locale::En => "Go",
            Locale::Cs => "Přejít",
            Locale::De => "Öffnen",
            Locale::Es => "Ir",
            Locale::Fr => "Aller",
            Locale::It => "Vai",
            Locale::Pl => "Idź",
            Locale::Pt => "Ir",
            Locale::Zh => "前往",
            Locale::Ja => "移動",
        }
    }

    pub fn menu_go_to_folder(&self) -> &'static str {
        self.modal_go_to_folder_title()
    }

    pub fn btn_disconnect(&self) -> &'static str {
        match self.locale {
            Locale::En => "Disconnect",
            Locale::Cs => "Odpojit",
            Locale::De => "Trennen",
            Locale::Es => "Desconectar",
            Locale::Fr => "Déconnecter",
            Locale::It => "Disconnetti",
            Locale::Pl => "Rozłącz",
            Locale::Pt => "Desconectar",
            Locale::Zh => "断开连接",
            Locale::Ja => "切断",
        }
    }

    pub fn btn_connect_server(&self) -> &'static str {
        match self.locale {
            Locale::En => "Connect to Server...",
            Locale::Cs => "Připojit k serveru...",
            Locale::De => "Mit Server verbinden...",
            Locale::Es => "Conectar al servidor...",
            Locale::Fr => "Se connecter au serveur...",
            Locale::It => "Connetti al server...",
            Locale::Pl => "Połącz z serwerem...",
            Locale::Pt => "Conectar ao servidor...",
            Locale::Zh => "连接到服务器...",
            Locale::Ja => "サーバーに接続...",
        }
    }

    pub fn menu_check_for_updates(&self) -> &'static str {
        match self.locale {
            Locale::En => "Check for Updates...",
            Locale::Cs => "Zkontrolovat aktualizace...",
            Locale::De => "Nach Updates suchen...",
            Locale::Es => "Buscar actualizaciones...",
            Locale::Fr => "Rechercher des mises à jour...",
            Locale::It => "Verifica aggiornamenti...",
            Locale::Pl => "Sprawdź dostępność aktualizacji...",
            Locale::Pt => "Verificar atualizações...",
            Locale::Zh => "检查更新...",
            Locale::Ja => "アップデートを確認...",
        }
    }

    pub fn update_title(&self) -> &'static str {
        match self.locale {
            Locale::En => "Software Update",
            Locale::Cs => "Aktualizace aplikace",
            Locale::De => "Software-Update",
            Locale::Es => "Actualización de software",
            Locale::Fr => "Mise à jour du logiciel",
            Locale::It => "Aggiornamento software",
            Locale::Pl => "Aktualizacja oprogramowania",
            Locale::Pt => "Atualização de software",
            Locale::Zh => "软件更新",
            Locale::Ja => "ソフトウェア・アップデート",
        }
    }

    pub fn update_available_msg(&self) -> &'static str {
        match self.locale {
            Locale::En => "A new version of MacSCP is available!",
            Locale::Cs => "Je k dispozici nová verze MacSCP!",
            Locale::De => "Eine neue Version von MacSCP ist verfügbar!",
            Locale::Es => "¡Hay una nueva versión de MacSCP disponible!",
            Locale::Fr => "Une nouvelle version de MacSCP est disponible !",
            Locale::It => "È disponibile una nuova versione di MacSCP!",
            Locale::Pl => "Dostępna jest nowa wersja MacSCP!",
            Locale::Pt => "Uma nova versão do MacSCP está disponível!",
            Locale::Zh => "MacSCP 有新版本可用！",
            Locale::Ja => "MacSCP の新しいバージョンが利用可能です！",
        }
    }

    pub fn update_uptodate_msg(&self) -> &'static str {
        match self.locale {
            Locale::En => "MacSCP is up to date.",
            Locale::Cs => "Máte nejnovější verzi MacSCP.",
            Locale::De => "MacSCP ist auf dem neuesten Stand.",
            Locale::Es => "MacSCP está actualizado.",
            Locale::Fr => "MacSCP est à jour.",
            Locale::It => "MacSCP è aggiornato.",
            Locale::Pl => "MacSCP jest aktualny.",
            Locale::Pt => "O MacSCP está atualizado.",
            Locale::Zh => "MacSCP 已是最新版本。",
            Locale::Ja => "MacSCP は最新の状態です。",
        }
    }

    pub fn update_brew_hint(&self) -> &'static str {
        match self.locale {
            Locale::En => "If you installed via Homebrew, run in Terminal:",
            Locale::Cs => "Pokud máte aplikaci z Homebrew, spusťte v Terminálu:",
            Locale::De => "Falls über Homebrew installiert, im Terminal ausführen:",
            Locale::Es => "Si instaló a través de Homebrew, ejecute en la Terminal:",
            Locale::Fr => "Si installé via Homebrew, exécutez dans le Terminal :",
            Locale::It => "Se installato tramite Homebrew, esegui nel Terminale:",
            Locale::Pl => "Jeśli zainstalowano przez Homebrew, uruchom w Terminalu:",
            Locale::Pt => "Se instalado via Homebrew, execute no Terminal:",
            Locale::Zh => "如果是通过 Homebrew 安装的，请在终端中运行：",
            Locale::Ja => "Homebrew でインストールした場合は、ターミナルで実行してください：",
        }
    }

    pub fn update_download_btn(&self) -> &'static str {
        match self.locale {
            Locale::En => "Download Release",
            Locale::Cs => "Stáhnout z GitHubu",
            Locale::De => "Release herunterladen",
            Locale::Es => "Descargar versión",
            Locale::Fr => "Télécharger la version",
            Locale::It => "Scarica versione",
            Locale::Pl => "Pobierz wydanie",
            Locale::Pt => "Baixar versão",
            Locale::Zh => "下载新版本",
            Locale::Ja => "リリースをダウンロード",
        }
    }

    pub fn update_copy_cmd(&self) -> &'static str {
        match self.locale {
            Locale::En => "Copy",
            Locale::Cs => "Kopírovat",
            Locale::De => "Kopieren",
            Locale::Es => "Copiar",
            Locale::Fr => "Copier",
            Locale::It => "Copia",
            Locale::Pl => "Kopiuj",
            Locale::Pt => "Copiar",
            Locale::Zh => "复制",
            Locale::Ja => "コピー",
        }
    }

    pub fn update_copied(&self) -> &'static str {
        match self.locale {
            Locale::En => "Copied!",
            Locale::Cs => "Zkopírováno!",
            Locale::De => "Kopiert!",
            Locale::Es => "¡Copiado!",
            Locale::Fr => "Copié !",
            Locale::It => "Copiato!",
            Locale::Pl => "Skopiowano!",
            Locale::Pt => "Copiado!",
            Locale::Zh => "已复制！",
            Locale::Ja => "コピーしました！",
        }
    }

    pub fn update_checking_msg(&self) -> &'static str {
        match self.locale {
            Locale::En => "Checking for updates...",
            Locale::Cs => "Kontroluji dostupnost aktualizací...",
            Locale::De => "Suche nach Updates...",
            Locale::Es => "Comprobando actualizaciones...",
            Locale::Fr => "Recherche de mises à jour...",
            Locale::It => "Verifica degli aggiornamenti...",
            Locale::Pl => "Sprawdzanie aktualizacji...",
            Locale::Pt => "Verificando atualizações...",
            Locale::Zh => "正在检查更新...",
            Locale::Ja => "アップデートを確認中...",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_codes_and_cycle() {
        assert_eq!(Locale::ALL.len(), 10);
        for &loc in Locale::ALL {
            let code = loc.as_code();
            assert_eq!(Locale::from_code(code), loc);
            assert!(!loc.display_name().is_empty());
            assert!(!loc.english_name().is_empty());
            assert!(!loc.flag().is_empty());
        }

        // Test cycle
        assert_eq!(Locale::En.next(), Locale::Cs);
        assert_eq!(Locale::Ja.next(), Locale::En);

        // Unknown code fallback
        assert_eq!(Locale::from_code("unknown"), Locale::En);

        // Test SSH config notice translations
        for &loc in Locale::ALL {
            let t = I18n::new(loc);
            assert!(!t.btn_import_ssh().is_empty());
            assert!(!t.btn_connecting().is_empty());
            assert!(!t.site_connecting_status("example.com", 22).is_empty());
            assert!(!t.notice_ssh_config_not_found().is_empty());
            assert!(!t.notice_ssh_config_no_hosts().is_empty());
            assert!(!t.notice_ssh_config_imported(1).is_empty());
            assert!(!t.notice_ssh_config_imported(3).is_empty());
            assert!(!t.notice_ssh_config_all_exist(2).is_empty());
        }
    }
}
