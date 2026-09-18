import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BinaryIcon` instead.
 *
 * @example
 * ```svelte
 * <Binary color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Binary: Component<IconComponentProps, {}, "">;
type Binary = ReturnType<typeof Binary>;
export default Binary;
