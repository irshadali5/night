# 🌙 Bihar Night (बिहार की निशा)
### The Contemplative Wayland Desktop Environment Palette & Cultural Inspiration

> *"Just as the nocturnal currents of the sacred Ganga reflect the quiet starlight of Pataliputra, and the sacred Bodhi tree sheltered the silent contemplation that sparked enlightenment, Bihar Night is crafted for the quiet hours of creation, thought, and digital craftsmanship."*

---

## 1. The Philosophy of Bihar Night

Most desktop themes draw their nocturnal aesthetic from the neon-lit cyberpunk skylines of megacities like Tokyo. While vibrant, that aesthetic represents sensory hyperactivity.

**Bihar Night** takes the opposite approach. Bihar is one of the world's most ancient cradles of philosophy, mathematics, astronomy, and spiritual awakening:

1. **Bodh Gaya & The Starlit Bodhi Tree**:
   Over 2,500 years ago, beneath the sprawling branches of the Mahabodhi tree on the banks of the Falgu river, Prince Siddhartha sat in meditative stillness through the quiet night watches. As the morning star appeared at dawn, he attained supreme Enlightenment (*Nirvana*). Bihar Night inherits this meditative serenity—an environment engineered to eliminate distraction, eye strain, and visual noise, allowing deep flow states during late-night coding.

2. **The Eternal Waters of River Ganga (Pataliputra / Patna)**:
   At dusk along the ghats of Patna, the atmosphere transforms. Thousands gather as golden brass oil lamps (*diyas*) and camphor fires dance during the Ganga Aarti, casting warm amber ripples across the deep obsidian water. Across the river, modern engineering wonders like the Mahatma Gandhi Setu and the Ganga Marine Drive illuminate the night horizon under super moons. The primary palette captures this sacred interplay between deep river midnight (`#16161e`) and the warm flicker of terracotta diya flames (`#e0af68`).

3. **Nalanda Mahavihara & Ancient Astronomy (Taregana)**:
   Nalanda was the world's first great residential university, housing thousands of scholars and millions of manuscripts. Just south of Patna lies Taregana (*"place of counting stars"*), the historic astronomical observatory where the great mathematician-astronomer **Aryabhata** studied the nocturnal skies, calculated the spherical earth, formulated the foundations of trigonometry, and established the profound mathematical concept of Zero (*Śūnya*). Bihar Night's geometry, clean ratios, and starlight typography honor this tradition of rigorous inquiry.

4. **Rajgir’s Mist-Cloaked Nocturnal Hills**:
   Surrounded by the five sacred hills of Ratnagiri and Vaibhargiri, ancient Rajagriha sits enveloped in evening mist and geothermal hot springs. The slate basalt stone of the hills, the cool bamboo groves of Venuvana, and the radiant white dome of the Vishwa Shanti Stupa in moonlight provide the foundation for the slate-storm variant.

5. **Mithila (Madhubani) Artisanal Heritage**:
   Centuries of Madhubani artistry relied solely on natural nocturnal pigments: deep indigo extracted from *Indigofera*, carbon lampblack (*kajal*) collected from mustard oil lamps, turmeric yellow, and warm terracotta ochre. Bihar Night translates these organic pigments into high-fidelity digital hex codes with high legibility and soft contrast.

---

## 2. Four Authentic Expressions of Bihar Night

The Bihar Night theme engine provides 4 specialized color variants tailored for different ambient lighting conditions and developer workflows:

| Variant Key | Bihar Name | Cultural Inspiration | Primary Accent | Base Background |
|:---|:---|:---|:---|:---|
| **`night`** | **Ganga Midnight** | River Ganga dusk, floating golden diyas, obsidian river currents | Golden Diya (`#e0af68`) & Ganga Blue (`#7aa2f7`) | Obsidian Water (`#16161e` / `#1a1b26`) |
| **`storm`** | **Rajgir Mist** | Ratnagiri slate basalt hills, bamboo mist, moonlit stupa | Mist White (`#c0caf5`) & Bamboo Leaf (`#9ece6a`) | Slate Basalt (`#1f2335` / `#24283b`) |
| **`moon`** | **Bodh Gaya Serenity** | Indigo starlight beneath the Mahabodhi Tree, brass temple bells | Starlight Indigo (`#82aaff`) & Temple Amber (`#ffc777`) | Bodhi Indigo (`#1e2030` / `#222436`) |
| **`day`** | **Nalanda Dawn** | Terracotta brick ruins and sandstone libraries at first light | Terracotta Red (`#f52a65`) & Ink Blue (`#3760bf`) | Sunlit Sandstone (`#e1e2e7` / `#d5d6db`) |

---

## 3. The Color Specifications

### Ganga Midnight (`night`) — Default
The flagship late-night dark theme. Soft on the eyes with deep contrast ratios exceeding WCAG AAA standards:
```toml
[colors.bihar_night_ganga]
bg_darkest     = "#16161e" # Deep Ganga nocturnal water
bg_primary     = "#1a1b26" # Calm ghat stone surface
bg_secondary   = "#24283b" # Elevated panel / window container
bg_highlight   = "#414868" # River reflection focus border
fg_primary     = "#c0caf5" # Constellation starlight
fg_muted       = "#a9b1d6" # Riverbank silt
fg_dark        = "#565f89" # Ancient stone inscriptions
accent_blue    = "#7aa2f7" # Falgu / Ganga deep current
accent_cyan    = "#7dcfff" # Clear nocturnal sky
accent_magenta = "#bb9af7" # Sacred lotus blossom
success_green  = "#9ece6a" # Lush riverbank groves
warning_yellow = "#e0af68" # Terracotta brass diya flame
error_red      = "#f7768e" # Sacred kumkum vermilion
```

### Rajgir Mist (`storm`)
Cooler slate-blue tones balancing mountain basalt with bamboo mist:
```toml
[colors.bihar_night_rajgir]
bg_darkest     = "#1f2335" # Ratnagiri hill shadow
bg_primary     = "#24283b" # Slate basalt foundation
bg_secondary   = "#292e42" # Misty hillside ridge
bg_highlight   = "#414868" # Stupa railing accent
fg_primary     = "#c0caf5" # Moonlit pagoda white
accent_blue    = "#7aa2f7" # Hot spring stream
success_green  = "#9ece6a" # Venuvana bamboo grove
warning_yellow = "#e0af68" # Hilltop beacon glow
```

### Bodh Gaya Serenity (`moon`)
Rich meditative indigo tuned for long hours of deep focus:
```toml
[colors.bihar_night_bodhgaya]
bg_darkest     = "#1e2030" # Midnight beneath the Bodhi tree
bg_primary     = "#222436" # Contemplation ground
bg_secondary   = "#2f334d" # Temple courtyard stone
bg_highlight   = "#444a73" # Mahabodhi spire silhouette
fg_primary     = "#c8d3f5" # Diamond Throne radiance
fg_muted       = "#82aaff" # Bodhi leaf reflection
accent_blue    = "#82aaff" # Sapphire mindfulness
accent_magenta = "#fca7ea" # Lotus petal offering
warning_yellow = "#ffc777" # Brass prayer wheel glow
```

### Nalanda Dawn (`day`)
Warm terracotta light theme inspired by Nalanda's sunlit ancient brick libraries:
```toml
[colors.bihar_night_nalanda]
bg_darkest     = "#d5d6db" # Aged parchment base
bg_primary     = "#e1e2e7" # Sun-warmed terracotta sandstone
bg_secondary   = "#e9e9ed" # Monastery corridor
bg_highlight   = "#c4c8da" # Sunlit column border
fg_primary     = "#3760bf" # Carbon palm-leaf manuscript ink
accent_blue    = "#2e7de9" # Clear morning sky
accent_magenta = "#9854f1" # Royal seal wax
success_green  = "#587539" # Ancient botanical garden
warning_yellow = "#8c6c3e" # Ochre monk robe
error_red      = "#f52a65" # Terracotta kiln red
```

---

## 4. Default Curated Wallpapers

Bihar Night includes high-resolution authentic photography packaged in `assets/wallpapers/`:

1. **`bihar-night.jpg` / `bihar-night-ganga.jpg`**:
   - **Scene**: River Ganga and Mahatma Gandhi Setu in Patna under the Super Moon Night.
   - **Resolution**: 5184 × 3456 (5K Ultra High Definition).
   - **Aesthetic**: Vast serene moonlit river horizon with soft golden bridge illumination reflecting across Ganga waters.

2. **`bihar-night-bodhgaya.jpg`**:
   - **Scene**: Mahabodhi Temple and Garden at Bodh Gaya illuminated at night.
   - **Resolution**: 4219 × 3164 (4K Ultra High Definition).
   - **Aesthetic**: Ancient sandstone Sikhara illuminated with golden light against the starlit sky, surrounded by tranquil temple gardens.

---

## 5. Usage & CLI Aliases

Switch variants anytime with pure Rust speed:

```bash
# Set Ganga Midnight (default dark)
nightctl theme set ganga
# or
nightctl theme set night

# Set Rajgir Mist
nightctl theme set rajgir
# or
nightctl theme set storm

# Set Bodh Gaya Serenity
nightctl theme set bodhgaya
# or
nightctl theme set moon

# Set Nalanda Dawn (light)
nightctl theme set nalanda
# or
nightctl theme set day

# Query active theme status & palette
nightctl theme status
```

Or open the **Bihar Night Control Center** directly via `Super + i` (`night-settings`).

