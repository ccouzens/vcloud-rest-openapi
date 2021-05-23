import { launch } from "puppeteer";
import { join } from "path";
import * as vm from "vm";

(async () => {
  const fileName = process.env.input;
  if (fileName === undefined) {
    throw new Error("Expected to receive `input` file name");
  }
  const browser = await launch();
  const page = await browser.newPage();
  await page.goto(`file://${join(process.cwd(), fileName)}`);

  const context: { defs?: any } = {};
  vm.createContext(context);
  vm.runInContext(
    await page.evaluate(
      () =>
        document.querySelector("body > script:first-of-type")?.textContent ?? ""
    ),
    context
  );
  const spec = {
    openapi: "3.1.0",
    info: {
      title: await page.evaluate(
        () => document.querySelector("title")?.textContent
      ),
      version: await page.evaluate(() =>
        document
          .querySelector("#generator .content")
          ?.textContent?.replace("Generated ", "")
          ?.trim()
      ),
    },
    components: {
      schemas: context.defs,
    },
  };

  await browser.close();

  console.log(JSON.stringify(spec, null, 2));
})().catch((e) => {
  console.error(e);
  process.exit(1);
});
