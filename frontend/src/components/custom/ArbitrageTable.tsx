import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";

type ArbitrageRow = {
  block: number;
  amountIn: number;
  amountOut: number;
  dex: string;
  path: string[];
};

const arbitrageData: ArbitrageRow[] = [
  {
    block: 2200022,
    amountIn: 0.0005,
    amountOut: 1.233242342,
    dex: "uniswap-v2",
    path: ["USDC", "DAI", "PEPE"],
  },
  {
    block: 2200023,
    amountIn: 0.0007,
    amountOut: 2.143242342,
    dex: "sushiswap",
    path: ["ETH", "USDT", "PEPE"],
  },
  {
    block: 2200024,
    amountIn: 0.0006,
    amountOut: 1.933242342,
    dex: "uniswap-v2",
    path: ["BNB", "BUSD", "DOGE"],
  },
];

function ArbitrageTable() {
  return (
    <div className="mx-auto border border-none rounded-3xl bg-zinc-900 p-6 dark:bg-muted transition-colors max-w-5xl font-mono">
      <Table>
        <TableCaption className="text-xs text-zinc-300 pb-4">
          Recent Arbitrage Opportunities
        </TableCaption>
        <TableHeader>
          <TableRow>
            <TableHead className="text-left text-stone-300 text-base font-bold pl-0 pb-3 pr-15">Block</TableHead>
            <TableHead className="text-left text-stone-300 text-base font-bold pl-0 pb-3 pr-15">Cost</TableHead>
            <TableHead className="text-left text-stone-300 text-base font-bold pl-0 pb-3 pr-15">Revenue</TableHead>
            <TableHead className="text-left text-stone-300 text-base font-bold pl-0 pb-3 pr-15">DEX</TableHead>
            <TableHead className="text-left text-stone-300 text-base font-bold pl-0 pb-3" >Path</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {arbitrageData.map((item, index) => (
            <TableRow
              key={index}
              className=""
            >
              <TableCell className="font-normal text-violet-400 pb-5 pt-5">
                {item.block}
              </TableCell>
              <TableCell className="text-stone-400  pl-0 pb-5 pt-5">
                ${item.amountIn.toFixed(4)}
              </TableCell>
              <TableCell className="text-stone-400 pl-0 pb-5 pt-5">
                ${item.amountOut.toFixed(6)}
              </TableCell>
              <TableCell className="pl-0 pb-5 pt-5">
                <Badge className="uppercase pl-0">
                  {item.dex}
                </Badge>
              </TableCell>
              <TableCell className="text-left space-x-1 pl-0 pb-5 pt-5">
                {item.path.map((token, i) => (
                  <Badge
                    key={i}
                    variant="secondary"
                    className="pl-0 py-0.5 text-xs  text-muted-foreground"
                  >
                    {token}
                  </Badge>
                ))}
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}

export default ArbitrageTable;
