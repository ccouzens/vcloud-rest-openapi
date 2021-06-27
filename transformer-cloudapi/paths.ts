import { Page } from "puppeteer";

export const paths = async (page: Page) =>
  await page.evaluate(() => {
    const pathItems: Record<
      string,
      {
        parameters: {
          name: string;
          in: "path";
          required: true;
          schema: { type: string };
        }[];
      }
    > = {};
    const pathAricles = document.querySelectorAll("#sections article");
    for (let i = 0; i < pathAricles.length; i++) {
      const pathArticle = pathAricles[i];
      const route =
        pathArticle.querySelector(
          "pre.prettyprint.language-html.prettyprinted span.pln"
        )?.textContent ?? null;
      if (route === null) {
        continue;
      }
      const parameters: {
        name: string;
        in: "path";
        required: true;
        schema: { type: string };
      }[] = [];
      const paramsTables = pathArticle.querySelectorAll("[id=methodsubtable]");
      for (let i = 0; i < paramsTables.length; i++) {
        const paramsTable = paramsTables[i];
        if (
          paramsTable.previousElementSibling?.textContent === "Path parameters"
        ) {
          const parameterRows = paramsTable.querySelectorAll("tr + tr");
          for (let i = 0; i < parameterRows.length; i++) {
            const parameterRow = parameterRows[i];
            const name =
              parameterRow.querySelector("td:first-child")?.textContent ?? null;
            const type =
              parameterRow
                .querySelector(".type")
                ?.textContent?.trim()
                ?.toLowerCase() ?? null;
            if (name === null || type === null) {
              continue;
            }
            parameters.push({
              name: name.split("*")[0],
              in: "path",
              required: true,
              schema: { type },
            });
          }
        }
      }
      pathItems[route] = { parameters };
    }
    return pathItems;
  });
