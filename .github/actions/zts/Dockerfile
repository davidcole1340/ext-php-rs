FROM php:zts

WORKDIR /tmp

RUN apt update -y && apt upgrade -y
RUN apt install git lsb-release wget gnupg software-properties-common -y
RUN bash -c "$(wget -O - https://apt.llvm.org/llvm.sh)"
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

COPY entrypoint.sh /tmp/entrypoint.sh
ENTRYPOINT ["/tmp/entrypoint.sh"]
