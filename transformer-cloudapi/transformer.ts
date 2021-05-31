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
  type Boolean = {
    type: "boolean";
    description?: string;
    default?: boolean;
  };
  type Integer = {
    type: "integer";
    description?: string;
    format?: "int32" | "int64";
    minimum?: number;
    maximum?: number;
  };
  type Number = {
    type: "number";
    description: string;
    format?: "double";
  };

  type String = {
    type: "string";
    format?: "date-time" | "password" | "uri";
    description?: string;
    example?: string;
    default?: string;
    minLength?: number;
    maxLength?: number;
    readOnly?: true;
  };
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
  };
  type DeepObject = {
    type: "object";
    description?: string;
    additionalProperties?:
      | {
          type: "string";
        }
      | Array;
  };
  type Object = {
    type?: "object";
    description?: string;
    properties: Record<
      string,
      Ref | Enum | Boolean | Integer | String | Number | Array | DeepObject
    >;
    required?: string[];
  };
  type Enum = {
    type: "object" | "string";
    description: string;
    default?: string;
    enum: string[];
  };

  const enumCorrector = (val: Enum): Enum => ({
    type: "string",
    description: val.description,
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
        : innerVal
    ),
    ...(outerVal.description !== undefined && {
      description: outerVal.description,
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
      newDefs[key] = value;
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
      schemas: await defs(page),
    },
  };

  await browser.close();

  console.log(JSON.stringify(spec, null, 2));
})().catch((e) => {
  console.error(e);
  process.exit(1);
});
