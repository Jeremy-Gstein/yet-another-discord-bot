FROM  rust:slim-bullseye

WORKDIR /usr/src/crabby

# Add ShodOS identifiers to /etc/lsb-release and /etc/os-release
RUN echo -e "DISTRIB_ID=ShodOS\nDISTRIB_RELEASE=1.0\nDISTRIB_CODENAME=shodo\nDISTRIB_DESCRIPTION='ShodOS Linux'" > /etc/lsb-release && \
    echo -e "NAME=\"ShodOS\"\nVERSION=\"1.0\"\nID=shodos\nID_LIKE=debian\nVERSION_ID=\"1.0\"\nPRETTY_NAME=\"ShodOS 1.0\"\nANSI_COLOR=\"0;34\"\nHOME_URL=\"https://null.com\"\nSUPPORT_URL=\"http://null.com/support\"\nBUG_REPORT_URL=\"http://null.com/bugs\"" > /etc/os-release && cargo install cargo-watch

COPY . .

CMD ["cargo", "watch", "-w", "src", "-x", "run"]
