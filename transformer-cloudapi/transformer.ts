import { launch, Page } from "puppeteer";
import { join } from "path";
import { paths } from "./paths";
import { defs } from "./defs";

(async () => {
  const fileName = process.env.input;
  if (fileName === undefined) {
    throw new Error("Expected to receive `input` file name");
  }
  const browser = await launch();
  const page = await browser.newPage();
  await page.goto(`file://${join(process.cwd(), fileName)}`);

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
      securitySchemes: {
        basicAuth: {
          type: "http",
          scheme: "basic",
        },
        bearerAuth: {
          type: "http",
          scheme: "bearer",
        },
      },
      schemas: await defs(page),
    },
    tags: await page.evaluate(() => {
      const headers = document.querySelectorAll(
        "#scrollingNav li.nav-header:not(.nav-fixed)"
      );
      const names = [];
      for (let i = 0; i < headers.length; i++) {
        names.push({ name: headers[i].getAttribute("data-group") });
      }
      return names;
    }),
    paths: await paths(page),
  };

  await browser.close();

  console.log(JSON.stringify(spec, null, 2));
})().catch((e) => {
  console.error(e);
  process.exit(1);
});
