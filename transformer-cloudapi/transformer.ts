import { launch } from "puppeteer";
import { join } from "path";

function generateSpec() {
  const spec = {
    openapi: "3.1.0",
    info: {
      title: document.querySelector("title")?.textContent,
      version: document
        .querySelector("#generator .content")
        ?.textContent?.replace("Generated ", "")
        ?.trim(),
    },
  };
  return spec;
}

(async () => {
  const fileName = process.env.input;
  if (fileName === undefined) {
    throw new Error("Expected to receive `input` file name");
  }
  const browser = await launch();
  const page = await browser.newPage();
  await page.goto(`file://${join(process.cwd(), fileName)}`);
  const spec = await page.evaluate(generateSpec);

  await browser.close();

  console.log(JSON.stringify(spec, null, 2));
})().catch((e) => {
  console.error(e);
  process.exit(1);
});
