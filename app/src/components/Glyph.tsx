// Inline SVG line icons used by the Misc row and the settings badge.

export type GlyphName = 'house' | 'gear' | 'inputs' | 'screensaver' | 'sliders' | 'wifi' | 'ethernet' | 'muted' | 'timer' | 'folder' | 'speaker' | 'picture' | 'power' | 'restart' | 'sun' | 'eye' | 'guide' | 'grid' | 'image' | 'cloud' | 'contrast' | 'globe' | 'trash' | 'minus' | 'list' | 'next';

export function Glyph ({name}: {name: GlyphName}) {
	return (
		<svg class="tile__glyph" viewBox="0 0 24 24" aria-hidden="true">
			{name === 'house'
				? <path d="M3 11.5 12 4l9 7.5M5.5 9.8V20h13V9.8M10 20v-6h4v6" />
				: name === 'wifi'
				? <path d="M2.5 9a14 14 0 0 1 19 0M5.6 12.5a9.5 9.5 0 0 1 12.8 0M8.8 16a5 5 0 0 1 6.4 0M12 19.4h.01" />
				: name === 'ethernet'
				? <path d="M4 6h16v8h-3v3.5h-3V14h-4v3.5H7V14H4zM8 9.2v2.2M12 9.2v2.2M16 9.2v2.2" />
				: name === 'muted'
				? <path d="M4 9.5h3.5L12 6v12l-4.5-3.5H4zM16 9.5l5 5M21 9.5l-5 5" />
				: name === 'timer'
				? <path d="M12 8v4.5l3 1.8M12 4.5a8 8 0 1 0 0 16 8 8 0 0 0 0-16zM9.5 2.5h5" />
				: name === 'speaker'
				? <path d="M4 9.5h3.5L12 6v12l-4.5-3.5H4zM15.5 9a4 4 0 0 1 0 6M18 6.5a8 8 0 0 1 0 11" />
				: name === 'picture'
				? <path d="M3.5 5.5h17v11h-17zM8 20h8M12 16.5V20" />
				: name === 'power'
				? <path d="M12 3.5v8M7 6.5a7.5 7.5 0 1 0 10 0" />
				: name === 'restart'
				? <path d="M19 12a7 7 0 1 1-2.2-5.1M19 4.5V8h-3.5" />
				: name === 'folder'
				? <path d="M3.5 7.5a2 2 0 0 1 2-2h4l2 2.5h7a2 2 0 0 1 2 2V17a2 2 0 0 1-2 2h-13a2 2 0 0 1-2-2z" />
				: name === 'trash'
				? <path d="M4.5 7h15M9.5 7V4.5h5V7M6.5 7l1 12.5h9l1-12.5M10 10.5v6M14 10.5v6" />
				: name === 'minus'
				? <path d="M12 3.5a8.5 8.5 0 1 1 0 17 8.5 8.5 0 0 1 0-17zM8 12h8" />
				: name === 'sliders'
				? <path d="M4 7h8M16 7h4M4 17h4M12 17h8M12 4.5v5M8 14.5v5" />
				: name === 'screensaver'
				? <path d="M20.5 14.2A8.5 8.5 0 0 1 9.8 3.5a8.5 8.5 0 1 0 10.7 10.7z" />
				: name === 'sun'
				? <path d="M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8zM12 2.5v2M12 19.5v2M2.5 12h2M19.5 12h2M5.3 5.3l1.4 1.4M17.3 17.3l1.4 1.4M5.3 18.7l1.4-1.4M17.3 6.7l1.4-1.4" />
				: name === 'eye'
				? <path d="M2.5 12S6 5.5 12 5.5 21.5 12 21.5 12 18 18.5 12 18.5 2.5 12 2.5 12zM12 9a3 3 0 1 0 0 6 3 3 0 0 0 0-6z" />
				: name === 'guide'
				? <path d="M3.5 5.5h17v13h-17zM3.5 9.5h17M9 9.5v9" />
				: name === 'list'
				? <path d="M8.5 6.5h12M8.5 12h12M8.5 17.5h12M4 6.5h.01M4 12h.01M4 17.5h.01" />
				: name === 'next'
				? <path d="M4 12h15M13.5 6.5 19 12l-5.5 5.5" />
				: name === 'grid'
				? <path d="M4 4h6.5v6.5H4zM13.5 4H20v6.5h-6.5zM4 13.5h6.5V20H4zM13.5 13.5H20V20h-6.5z" />
				: name === 'image'
				? <path d="M3.5 5h17v14h-17zM3.5 15.5l5-5 4 4 3-3 5 5M15.5 9.3a.8.8 0 1 0 0-.1" />
				: name === 'cloud'
				? <path d="M7.5 19h10a4 4 0 0 0 .4-8 6 6 0 0 0-11.6 1.6A3.3 3.3 0 0 0 7.5 19z" />
				: name === 'contrast'
				? <path d="M12 3.5a8.5 8.5 0 1 0 0 17 8.5 8.5 0 0 0 0-17zM12 3.5v17M12 7.5h4.5M12 11.5h6.5M12 15.5h5" />
				: name === 'globe'
				? <path d="M12 3.5a8.5 8.5 0 1 0 0 17 8.5 8.5 0 0 0 0-17zM3.5 12h17M12 3.5c-2.6 2.4-3.8 5.2-3.8 8.5s1.2 6.1 3.8 8.5c2.6-2.4 3.8-5.2 3.8-8.5S14.6 5.9 12 3.5z" />
				: name === 'inputs'
				? <path d="M3.5 7.5h17v7.2a2 2 0 0 1-2 2h-3.2l-1 2.3H9.7l-1-2.3H5.5a2 2 0 0 1-2-2zM8 10.5v2.4M12 10.5v2.4M16 10.5v2.4" />
				: <path d="M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z" />}
		</svg>
	);
}

export function CheckMark () {
	return (
		<svg viewBox="0 0 24 24" aria-hidden="true">
			<path d="M5 12.5l4.5 4.5L19 7.5" />
		</svg>
	);
}
