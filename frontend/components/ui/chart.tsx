import { Line } from "recharts";
import { Bar } from "recharts";
import { XAxis } from "recharts";
import { YAxis } from "recharts";
import { CartesianGrid } from "recharts";
import { Tooltip } from "recharts";
import { Legend } from "recharts";
import { ResponsiveContainer } from "recharts";
import { ComposedChart } from "recharts";

type LineChartProps = {
  data: any[];
  index: string;
  categories: string[];
  colors: string[];
  yAxisWidth: number;
  valueFormatter: (value: number) => string;
};

export function LineChart({
  data,
  index,
  categories,
  colors,
  yAxisWidth,
  valueFormatter,
}: LineChartProps) {
  return (
    <ResponsiveContainer width="100%" height={300}>
      <ComposedChart
        data={data}
        margin={{ top: 20, right: 20, bottom: 20, left: yAxisWidth }}
      >
        <CartesianGrid strokeDasharray="3 3" />
        <XAxis dataKey={index} />
        <YAxis width={yAxisWidth} />
        <Tooltip formatter={valueFormatter} />
        <Legend />
        {categories.map((category, index) => (
          <Line
            key={index}
            type="monotone"
            dataKey={category}
            stroke={colors[index]}
          />
        ))}
      </ComposedChart>
    </ResponsiveContainer>
  );
}

type BarChartProps = {
  data: any[];
  index: string;
  categories: string[];
  colors: string[];
  yAxisWidth: number;
};

export function BarChart({
  data,
  index,
  categories,
  colors,
  yAxisWidth,
}: BarChartProps) {
  return (
    <ResponsiveContainer width="100%" height={300}>
      <ComposedChart
        data={data}
        margin={{ top: 20, right: 20, bottom: 20, left: yAxisWidth }}
      >
        <CartesianGrid strokeDasharray="3 3" />
        <XAxis dataKey={index} />
        <YAxis width={yAxisWidth} />
        <Tooltip />
        <Legend />
        {categories.map((category, index) => (
          <Bar
            key={index}
            dataKey={category}
            fill={colors[index]}
            barSize={20}
          />
        ))}
      </ComposedChart>
    </ResponsiveContainer>
  );
}
