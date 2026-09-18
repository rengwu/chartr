import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NotePencilIcon` instead.
 *
 * @example
 * ```svelte
 * <NotePencil color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NotePencil: Component<IconComponentProps, {}, "">;
type NotePencil = ReturnType<typeof NotePencil>;
export default NotePencil;
