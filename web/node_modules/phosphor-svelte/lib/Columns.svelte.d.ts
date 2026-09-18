import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ColumnsIcon` instead.
 *
 * @example
 * ```svelte
 * <Columns color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Columns: Component<IconComponentProps, {}, "">;
type Columns = ReturnType<typeof Columns>;
export default Columns;
