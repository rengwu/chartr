import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `WrenchIcon` instead.
 *
 * @example
 * ```svelte
 * <Wrench color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Wrench: Component<IconComponentProps, {}, "">;
type Wrench = ReturnType<typeof Wrench>;
export default Wrench;
