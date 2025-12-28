import { LucideProps } from "lucide-react";

type Props = {
    title: string;
    Icon: React.FC<LucideProps>;
    children: React.ReactNode;
};

export const SectionTitle: React.FC<Props> = ({ title, Icon, children }) => {
    return (
        <div className="flex flex-col gap-2">
            <div className="flex items-center gap-3">
                <Icon className="text-primary" size={21} />
                <h6 className="text-medium font-medium">{title}</h6>
            </div>

            <div className="flex flex-col gap-4">{children}</div>
        </div>
    );
};
