import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PlusCircleIcon` instead.
 *
 * @example
 * ```svelte
 * <PlusCircle color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PlusCircle: Component<IconComponentProps, {}, "">;
type PlusCircle = ReturnType<typeof PlusCircle>;
export default PlusCircle;
