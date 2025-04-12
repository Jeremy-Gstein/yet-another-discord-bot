FROM  rust:bookworm

WORKDIR /usr/src/bday

# Add ShodOS identifiers to /etc/lsb-release and /etc/os-release
RUN  echo -e "NAME=\"ShodOS\"\nVERSION=\"1.0\"\nID=shodos\nID_LIKE=debian\nVERSION_ID=\"1.0\"\nPRETTY_NAME=\"ShodOS 1.0\"\nANSI_COLOR=\"0;34\"\nHOME_URL=\"https://null.com\"\nSUPPORT_URL=\"http://null.com/support\"\nBUG_REPORT_URL=\"http://null.com/bugs\"" > /etc/os-release && cargo install cargo-watch

COPY . .

CMD ["cargo", "watch", "-w", "src", "-x", "run"]
