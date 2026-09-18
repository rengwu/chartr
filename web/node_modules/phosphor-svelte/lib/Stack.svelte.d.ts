import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `StackIcon` instead.
 *
 * @example
 * ```svelte
 * <Stack color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Stack: Component<IconComponentProps, {}, "">;
type Stack = ReturnType<typeof Stack>;
export default Stack;
