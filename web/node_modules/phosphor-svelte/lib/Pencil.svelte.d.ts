import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PencilIcon` instead.
 *
 * @example
 * ```svelte
 * <Pencil color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Pencil: Component<IconComponentProps, {}, "">;
type Pencil = ReturnType<typeof Pencil>;
export default Pencil;
