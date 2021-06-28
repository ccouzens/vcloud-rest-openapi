import { Page } from "puppeteer";

type ParameterType = {
  name: string;
  in: "path";
  required: true;
  schema: { type: string };
};

type OperationObjectType = {
  tags: string[];
  description: string;
};

type PathItemType = {
  parameters: ParameterType[];
  get?: OperationObjectType;
  put?: OperationObjectType;
  post?: OperationObjectType;
  delete?: OperationObjectType;
};

export const paths = async (page: Page) =>
  await page.evaluate(() => {
    function populateParams(pathArticle: Element, pathItem: PathItemType) {
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
            pathItem.parameters.push({
              name: name.split("*")[0],
              in: "path",
              required: true,
              schema: { type },
            });
          }
        }
      }
    }

    const pathItems: Record<string, PathItemType> = {};
    const pathAricles = document.querySelectorAll("#sections article");
    for (let i = 0; i < pathAricles.length; i++) {
      const pathArticle = pathAricles[i];
      const routeElement = pathArticle.querySelector(
        "pre.prettyprint.language-html.prettyprinted span.pln"
      );
      if (routeElement === null) {
        continue;
      }
      const route = routeElement.textContent ?? null;
      if (route === null) {
        continue;
      }

      const pathItem: PathItemType = pathItems[route] ?? { parameters: [] };
      pathItems[route] = pathItem;

      if (pathItem.parameters.length === 0) {
        populateParams(pathArticle, pathItem);
      }

      const description =
        pathArticle.querySelector("div")?.textContent?.trim() ?? null;
      if (description === null) {
        continue;
      }
      const operationObject: OperationObjectType = { tags: [], description };
      switch (
        routeElement.parentElement?.parentElement?.getAttribute("data-type")
      ) {
        case "get":
          pathItem.get = operationObject;
          break;
        case "put":
          pathItem.put = operationObject;
          break;
        case "post":
          pathItem.post = operationObject;
          break;
        case "delete":
          pathItem.delete = operationObject;
          break;
      }
    }
    return pathItems;
  });
