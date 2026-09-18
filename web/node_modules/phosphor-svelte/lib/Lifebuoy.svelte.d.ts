import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LifebuoyIcon` instead.
 *
 * @example
 * ```svelte
 * <Lifebuoy color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Lifebuoy: Component<IconComponentProps, {}, "">;
type Lifebuoy = ReturnType<typeof Lifebuoy>;
export default Lifebuoy;
