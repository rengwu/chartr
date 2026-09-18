import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `RankingIcon` instead.
 *
 * @example
 * ```svelte
 * <Ranking color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Ranking: Component<IconComponentProps, {}, "">;
type Ranking = ReturnType<typeof Ranking>;
export default Ranking;
