import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BiohazardIcon` instead.
 *
 * @example
 * ```svelte
 * <Biohazard color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Biohazard: Component<IconComponentProps, {}, "">;
type Biohazard = ReturnType<typeof Biohazard>;
export default Biohazard;
