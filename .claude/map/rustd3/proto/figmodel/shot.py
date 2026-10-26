import sys, pathlib
from playwright.sync_api import sync_playwright
with sync_playwright() as p:
    b = p.chromium.launch(args=["--disable-gpu", "--no-sandbox"])
    pg = b.new_page(viewport={"width": 760, "height": 520})
    for f in sys.argv[1:]:
        pg.goto(pathlib.Path(f).resolve().as_uri())
        pg.screenshot(path=f.replace(".svg", ".png"))
    b.close()
