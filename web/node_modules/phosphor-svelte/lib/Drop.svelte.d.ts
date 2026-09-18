import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DropIcon` instead.
 *
 * @example
 * ```svelte
 * <Drop color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Drop: Component<IconComponentProps, {}, "">;
type Drop = ReturnType<typeof Drop>;
export default Drop;
