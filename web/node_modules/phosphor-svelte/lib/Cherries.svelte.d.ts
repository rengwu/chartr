import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CherriesIcon` instead.
 *
 * @example
 * ```svelte
 * <Cherries color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Cherries: Component<IconComponentProps, {}, "">;
type Cherries = ReturnType<typeof Cherries>;
export default Cherries;
