import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FileCsvIcon` instead.
 *
 * @example
 * ```svelte
 * <FileCsv color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FileCsv: Component<IconComponentProps, {}, "">;
type FileCsv = ReturnType<typeof FileCsv>;
export default FileCsv;
