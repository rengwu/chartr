import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CrossIcon` instead.
 *
 * @example
 * ```svelte
 * <Cross color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Cross: Component<IconComponentProps, {}, "">;
type Cross = ReturnType<typeof Cross>;
export default Cross;
