import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `StackPlusIcon` instead.
 *
 * @example
 * ```svelte
 * <StackPlus color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const StackPlus: Component<IconComponentProps, {}, "">;
type StackPlus = ReturnType<typeof StackPlus>;
export default StackPlus;
