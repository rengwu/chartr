import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CactusIcon` instead.
 *
 * @example
 * ```svelte
 * <Cactus color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Cactus: Component<IconComponentProps, {}, "">;
type Cactus = ReturnType<typeof Cactus>;
export default Cactus;
