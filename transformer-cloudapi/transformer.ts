import { launch, Page } from "puppeteer";
import { join } from "path";
import * as vm from "vm";

async function defs(page: Page) {
  const script = await page.evaluate(
    () =>
      document.querySelector("body > script:first-of-type")?.textContent ?? ""
  );
  if (script === undefined) {
    throw new Error("Expected to read javascript to create defs");
  }
  type Ref = {
    $ref: string;
    description?: string;
  };

  const refStringCorrector = (val: string): string =>
    val.replace(/^\#\/definitions\//, "#/components/schemas/");

  const refCorrector = (val: Ref): Ref => ({
    $ref: refStringCorrector(val.$ref),
    ...(val.description !== undefined && {
      description: descriptionCorrector(val.description),
    }),
  });

  const descriptionCorrector = (val: string): string => val.trim();

  type Boolean = {
    type: "boolean";
    description?: string;
    default?: boolean;
  };

  const booleanCorrector = (val: Boolean): Boolean => ({
    type: "boolean",
    ...(val.description !== undefined && {
      description: descriptionCorrector(val.description),
    }),
    ...(val.default !== undefined && { default: val.default }),
  });

  type Integer = {
    type: "integer";
    description?: string;
    format?: "int32" | "int64";
    minimum?: number;
    maximum?: number;
    default?: number;
    readOnly?: boolean;
  };

  const integerCorrector = (val: Integer): Integer => ({
    type: "integer",
    ...((val.format === "int32" || val.format === "int64") && {
      format: val.format,
    }),
    ...(val.description !== undefined && {
      description: descriptionCorrector(val.description),
    }),
    ...(val.minimum !== undefined && { minimum: val.minimum }),
    ...(val.maximum !== undefined && { maximum: val.maximum }),
    ...(val.default !== undefined && { default: val.default }),
    ...(val.readOnly !== undefined && { readOnly: val.readOnly }),
  });

  type Number = {
    type: "number";
    description?: string;
    format?: "double";
    minimum?: number;
    maximum?: number;
  };

  const numberCorrector = (val: Number): Number => ({
    type: "number",
    ...(val.format === "double" && {
      format: val.format,
    }),
    ...(val.description !== undefined && {
      description: descriptionCorrector(val.description),
    }),
    ...(val.minimum !== undefined && { minimum: val.minimum }),
    ...(val.maximum !== undefined && { maximum: val.maximum }),
  });

  type String = {
    type: "string";
    format?: "date-time" | "password" | "uri";
    description?: string;
    example?: string;
    examples?: string[];
    default?: string;
    minLength?: number;
    maxLength?: number;
    readOnly?: true;
    pattern?: string;
  };

  const stringCorrector = (val: String): String => ({
    type: "string",
    ...((val.format === "date-time" || val.format === "uri") && {
      format: val.format,
    }),
    ...(val.description !== undefined && {
      description: descriptionCorrector(val.description),
    }),
    ...(val.example !== undefined && {
      examples: [val.example],
    }),
    ...(val.default !== undefined && { default: val.default }),
    ...(val.minLength !== undefined && { minLength: val.minLength }),
    ...(val.maxLength !== undefined && { maxLength: val.maxLength }),
    ...(val.readOnly !== undefined && { readOnly: val.readOnly }),
    ...(val.pattern !== undefined && { pattern: val.pattern }),
  });

  type Array = {
    type: "array";
    description?: string;
    items:
      | {
          $ref: string;
        }
      | {
          type: "string";
        };
    minItems?: number;
    maxItems?: number;
  };
  const arrayCorrector = (val: Array): Array => ({
    type: "array",
    ...(val.description !== undefined && {
      description: descriptionCorrector(val.description),
    }),
    items: {
      ...("$ref" in val.items
        ? { $ref: refStringCorrector(val.items.$ref) }
        : { type: "string" }),
    },
    ...(val.minItems !== undefined && { minItems: val.minItems }),
    ...(val.maxItems !== undefined && { maxItems: val.maxItems }),
  });

  type DeepObject = {
    type: "object";
    description?: string;
    additionalProperties?:
      | {
          type: "string";
        }
      | Array;
  };

  const deepObjectCorrector = (val: DeepObject): DeepObject => ({
    type: "object",
    ...(val.description !== undefined && {
      description: descriptionCorrector(val.description),
    }),
    ...(val.additionalProperties !== undefined && {
      additionalProperties: {
        ...(val.additionalProperties.type === "string"
          ? { type: "string" }
          : arrayCorrector(val.additionalProperties)),
      },
    }),
  });

  type Object = {
    type?: "object";
    description?: string;
    properties?: Record<
      string,
      Ref | Enum | Boolean | Integer | String | Number | Array | DeepObject
    >;
    required?: string[];
    discriminator?: string;
  };

  const objectCorrector = (val: Object): Object => ({
    type: "object",
    ...(val.description !== undefined && {
      description: descriptionCorrector(val.description),
    }),
    ...(val.properties !== undefined && {
      properties: Object.entries(val.properties)
        .map(([key, value]): Object["properties"] => {
          if ("$ref" in value) {
            return { [key]: refCorrector(value) };
          } else if ("enum" in value) {
            return { [key]: enumCorrector(value) };
          } else if (value.type === "boolean") {
            return { [key]: booleanCorrector(value) };
          } else if (value.type == "integer") {
            return { [key]: integerCorrector(value) };
          } else if (value.type === "number") {
            return { [key]: numberCorrector(value) };
          } else if (value.type === "string") {
            return { [key]: stringCorrector(value) };
          } else if (value.type === "array") {
            return { [key]: arrayCorrector(value) };
          } else if (value.type === "object") {
            return { [key]: deepObjectCorrector(value) };
          } else {
            throw new Error("Unexpected object type");
          }
        })
        .reduce(
          (previousValue, currentValue) => ({
            ...previousValue,
            ...currentValue,
          }),
          {}
        ),
    }),
    ...(val.required !== undefined && {
      required: val.required,
    }),
    ...(val.discriminator !== undefined && {
      discriminator: val.discriminator,
    }),
  });

  type Enum = {
    type: "object" | "string";
    description?: string;
    default?: string;
    enum: string[];
  };

  const enumCorrector = (val: Enum): Enum => ({
    type: "string",
    ...(val.description !== undefined && {
      description: descriptionCorrector(val.description),
    }),
    enum: val.enum,
    ...(val.default !== undefined && { default: val.default }),
  });

  type AllOf = {
    allOf: ({ $ref: string } | Object)[];
    description?: string;
  };

  const allOfCorrector = (outerVal: AllOf): AllOf => ({
    allOf: outerVal.allOf.map((innerVal) =>
      "$ref" in innerVal
        ? { $ref: refStringCorrector(innerVal.$ref) }
        : objectCorrector(innerVal)
    ),
    ...(outerVal.description !== undefined && {
      description: descriptionCorrector(outerVal.description),
    }),
  });

  const context: { defs?: Record<string, Object | Enum | AllOf> } = {};
  vm.createContext(context);
  vm.runInContext(script, context);

  const defs = context.defs;

  if (defs === undefined) {
    throw new Error("Expected to get `defs` from page");
  }

  const newDefs: Record<string, Enum | Object | AllOf> = {};

  for (const [key, value] of Object.entries(defs)) {
    if ("enum" in value) {
      newDefs[key] = enumCorrector(value);
    } else if ("allOf" in value) {
      newDefs[key] = allOfCorrector(value);
    } else {
      newDefs[key] = objectCorrector(value);
    }
  }

  return newDefs;
}

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
    paths: await page.evaluate(() => {
      const paths: Record<string, {}> = {};
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
        const paramsTables = pathArticle.querySelectorAll(
          "[id=methodsubtable]"
        );
        for (let i = 0; i < paramsTables.length; i++) {
          const paramsTable = paramsTables[i];
          if (
            paramsTable.previousElementSibling?.textContent ===
            "Path parameters"
          ) {
            const parameterRows = paramsTable.querySelectorAll("tr + tr");
            for (let i = 0; i < parameterRows.length; i++) {
              const parameterRow = parameterRows[i];
              const name =
                parameterRow.querySelector("td:first-child")?.textContent ??
                null;
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
        paths[route] = { parameters };
      }
      return paths;
    }),
  };

  await browser.close();

  console.log(JSON.stringify(spec, null, 2));
})().catch((e) => {
  console.error(e);
  process.exit(1);
});
