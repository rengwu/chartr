import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CardsThreeIcon` instead.
 *
 * @example
 * ```svelte
 * <CardsThree color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CardsThree: Component<IconComponentProps, {}, "">;
type CardsThree = ReturnType<typeof CardsThree>;
export default CardsThree;
