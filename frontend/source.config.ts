import { defineDocs, frontmatterSchema, metaSchema } from "fumadocs-mdx/config";

export const docs = defineDocs({
  dir: "content/docs",
  docs: {
    schema: frontmatterSchema.extend({}),
  },
  meta: {
    schema: metaSchema,
  },
});
