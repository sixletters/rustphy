# GitHub Pages Setup Guide

This guide will help you publish your Rustphy project and MIT lecture series on GitHub Pages.

## Files Created

I've created the following files for GitHub Pages:

- `_config.yml` - Jekyll configuration with theme and settings
- `index.md` - Main landing page (MIT lecture series)

## Step-by-Step Setup

### 1. Update `_config.yml`

Before pushing, update these values in `_config.yml`:

```yaml
url: "https://YOUR_USERNAME.github.io"  # Replace with your GitHub username
baseurl: "/rustphy"                      # Keep this if repo name is rustphy

# Update navigation URLs if needed
```

### 2. Push to GitHub

```bash
git add .
git commit -m "Add GitHub Pages configuration"
git push origin main
```

### 3. Enable GitHub Pages

1. Go to your repository on GitHub: `https://github.com/YOUR_USERNAME/rustphy`
2. Click **Settings** (top right)
3. Scroll down to **Pages** (left sidebar, under "Code and automation")
4. Under **Source**, select:
   - Branch: `main`
   - Folder: `/ (root)`
5. Click **Save**

### 4. Wait for Deployment

- GitHub will take 1-2 minutes to build your site
- You'll see a green checkmark when it's ready
- Your site will be available at: `https://YOUR_USERNAME.github.io/rustphy/`

### 5. Check Your Site

Visit your site at:
- **Home (Lecture Series):** `https://YOUR_USERNAME.github.io/rustphy/`
- **Lecture 1:** `https://YOUR_USERNAME.github.io/rustphy/learning/MIT_opencourseware/lecture_1_intro/data.html`

## Customization Options

### Change Theme

Edit `_config.yml` and try different themes:

```yaml
theme: jekyll-theme-cayman      # Current (clean, modern)
# theme: jekyll-theme-minimal   # Minimalist sidebar
# theme: jekyll-theme-slate     # Dark theme
# theme: jekyll-theme-architect # Header-focused
# theme: jekyll-theme-hacker    # Terminal-style
```

### Add Custom Domain (Optional)

If you have a custom domain:

1. In repository Settings → Pages
2. Enter your domain under **Custom domain**
3. Add a `CNAME` file to your repo root with your domain name

## File Structure

```
rustphy/
├── _config.yml                  # Jekyll configuration
├── index.md                     # Home page (MIT lecture series landing)
└── learning/
    └── MIT_opencourseware/
        └── lecture_1_intro/
            └── data.md          # Lecture 1 content
```

**Note:** Only the `learning/` directory and `index.md` are published. All Rust project files are excluded.

## Updating Content

When you add new lectures:

1. Create a new folder: `learning/MIT_opencourseware/lecture_2_lexical/`
2. Add your markdown file: `data.md`
3. Update `index.md` in the root to link to it
4. Commit and push - GitHub Pages auto-rebuilds!

```bash
git add learning/MIT_opencourseware/lecture_2_lexical/
git add index.md
git commit -m "Add Lecture 2: Lexical Analysis"
git push
```

## Troubleshooting

### Site Not Showing Up?

- Check Settings → Pages shows "Your site is live at..."
- Wait 2-3 minutes after pushing
- Check the Actions tab for build errors

### Markdown Not Rendering?

- Ensure files have `.md` extension
- Check frontmatter is correct:
  ```yaml
  ---
  layout: default
  title: Your Title
  ---
  ```

### Links Broken?

- Use relative links: `[Lecture 1](lecture_1_intro/data.md)`
- Not absolute: `[Lecture 1](/rustphy/learning/...)`

### Code Not Highlighting?

- GitHub Pages uses Rouge for syntax highlighting
- Use proper code fences with language:
  ````markdown
  ```c
  int main() { return 0; }
  ```
  ````

## Resources

- [GitHub Pages Documentation](https://docs.github.com/en/pages)
- [Jekyll Themes](https://pages.github.com/themes/)
- [Markdown Guide](https://www.markdownguide.org/)

## Next Steps

After your site is live:

1. ✅ Share the link on social media
2. ✅ Add the URL to your GitHub repo description
3. ✅ Continue adding lectures
4. ✅ Get feedback from readers!

---

**Your site will be at:** `https://YOUR_USERNAME.github.io/rustphy/`

Replace `YOUR_USERNAME` with your actual GitHub username!
